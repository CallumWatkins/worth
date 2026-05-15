use serde::{Deserialize, Serialize};
use worker::*;

const GITHUB_OWNER: &str = "CallumWatkins";
const GITHUB_REPO: &str = "worth";
const RELEASE_METADATA_CACHE_URL: &str =
    "https://releases.useworth.app/__cache/github-release-latest";
const LATEST_JSON_CACHE_PATH: &str = "/__cache/latest-json";
const LATEST_JSON_TTL_SECONDS: u32 = 60;
const RELEASE_METADATA_TTL_SECONDS: u32 = 300;
const STABLE_DOWNLOAD_REDIRECT_TTL_SECONDS: u32 = 300;
const SPECIFIC_DOWNLOAD_REDIRECT_TTL_SECONDS: u32 = 3600;

const WEBSITE_DOWNLOADS: &[StableDownloadDescriptor] = &[
    StableDownloadDescriptor {
        id: "windows",
        label: "Windows",
        kind: "exe",
    },
    StableDownloadDescriptor {
        id: "macos-aarch64",
        label: "macOS Apple Silicon",
        kind: "dmg",
    },
    StableDownloadDescriptor {
        id: "macos-x86_64",
        label: "macOS Intel",
        kind: "dmg",
    },
    StableDownloadDescriptor {
        id: "linux",
        label: "Linux",
        kind: "AppImage",
    },
];

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
}

#[derive(Serialize)]
struct StableManifest {
    version: String,
    downloads: Vec<StableManifestDownload>,
}

#[derive(Serialize)]
struct StableManifestDownload {
    id: &'static str,
    label: &'static str,
    href: String,
    kind: &'static str,
}

struct StableDownloadDescriptor {
    id: &'static str,
    label: &'static str,
    kind: &'static str,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/health", |_req, _ctx| health())
        .get_async(
            "/v1/update/stable/:target/:arch/:currentVersion",
            |req, ctx| async move { stable_update(req, ctx).await },
        )
        .get_async("/v1/stable.json", |req, ctx| async move {
            stable_manifest(req, ctx).await
        })
        .get_async("/v1/download/stable/:platform", |req, ctx| async move {
            stable_download(req, ctx).await
        })
        .head_async("/v1/download/stable/:platform", |req, ctx| async move {
            stable_download(req, ctx).await
        })
        .get_async("/v1/download/:version/:filename", |_req, ctx| async move {
            specific_download(ctx)
        })
        .head_async("/v1/download/:version/:filename", |_req, ctx| async move {
            specific_download(ctx)
        })
        .run(req, env)
        .await
}

fn health() -> Result<Response> {
    json_response(&HealthResponse { ok: true }, Some(0))
}

async fn stable_update(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    cached_updater_manifest_response(releases_base_url(&req)?)
        .await
        .or_else(|error| upstream_unavailable(&error))
}

async fn stable_manifest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let base_url = releases_base_url(&req)?;

    match latest_release(&ctx).await {
        Ok(release) => {
            let downloads = WEBSITE_DOWNLOADS
                .iter()
                .filter(|download| find_stable_asset(&release, download.id).is_some())
                .map(|download| {
                    stable_platform_download_url(&base_url, download.id).map(|href| {
                        StableManifestDownload {
                            id: download.id,
                            label: download.label,
                            href,
                            kind: download.kind,
                        }
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            let manifest = StableManifest {
                version: display_version(&release.tag_name).to_string(),
                downloads,
            };

            json_response(&manifest, Some(RELEASE_METADATA_TTL_SECONDS))
        }
        Err(error) => upstream_unavailable(&error),
    }
}

async fn stable_download(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let platform = match decoded_route_param(&ctx, "platform") {
        Ok(value) => value,
        Err(_) => return text_error("Invalid stable platform", 400),
    };

    match latest_release(&ctx).await {
        Ok(release) => match find_stable_asset(&release, &platform) {
            Some(asset) => redirect_response(
                versioned_download_url(releases_base_url(&req)?, &release.tag_name, &asset.name)?,
                STABLE_DOWNLOAD_REDIRECT_TTL_SECONDS,
            ),
            None => text_error(
                format!("No stable release asset found for platform `{platform}`"),
                404,
            ),
        },
        Err(error) => upstream_unavailable(&error),
    }
}

fn specific_download(ctx: RouteContext<()>) -> Result<Response> {
    let version = match decoded_route_param(&ctx, "version") {
        Ok(value) => value,
        Err(_) => return text_error("Invalid release version", 400),
    };
    let filename = match decoded_route_param(&ctx, "filename") {
        Ok(value) => value,
        Err(_) => return text_error("Invalid release filename", 400),
    };

    if !validate_version(&version) {
        return text_error("Invalid release version", 400);
    }

    if !validate_filename(&filename) {
        return text_error("Invalid release filename", 400);
    }

    redirect_response(
        github_download_url(&version, &filename)?,
        SPECIFIC_DOWNLOAD_REDIRECT_TTL_SECONDS,
    )
}

async fn latest_release(ctx: &RouteContext<()>) -> Result<GitHubRelease> {
    let text = cached_github_api_text(
        ctx,
        RELEASE_METADATA_CACHE_URL,
        &format!("https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest"),
        RELEASE_METADATA_TTL_SECONDS,
    )
    .await?;

    serde_json::from_str(&text).map_err(Error::from)
}

async fn cached_updater_manifest_response(base_url: Url) -> Result<Response> {
    let cache = Cache::default();
    let cache_key_url = cache_url(&base_url, LATEST_JSON_CACHE_PATH)?;
    let cache_key = Request::new(&cache_key_url, Method::Get)?;

    if let Some(response) = cache.get(&cache_key, false).await? {
        return Ok(response);
    }

    let text = fetch_public_asset_text(&format!(
        "https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest/download/latest.json"
    ))
    .await?;
    let text = rewrite_update_manifest(&text, &base_url)?;
    let mut response = json_text_response(text, LATEST_JSON_TTL_SECONDS)?;

    cache.put(&cache_key, response.cloned()?).await?;

    Ok(response)
}

async fn cached_github_api_text(
    ctx: &RouteContext<()>,
    cache_url: &str,
    url: &str,
    ttl_seconds: u32,
) -> Result<String> {
    let cache = Cache::default();
    let cache_key = Request::new(cache_url, Method::Get)?;

    if let Some(mut response) = cache.get(&cache_key, false).await? {
        return response.text().await;
    }

    let text = fetch_github_api_text(ctx, url).await?;
    let mut response = json_text_response(text.clone(), ttl_seconds)?;

    cache.put(&cache_key, response.cloned()?).await?;

    Ok(text)
}

async fn fetch_github_api_text(ctx: &RouteContext<()>, url: &str) -> Result<String> {
    let headers = Headers::new();
    headers.set("Accept", "application/vnd.github+json")?;
    headers.set("User-Agent", "worth-releases-worker")?;
    headers.set("X-GitHub-Api-Version", "2022-11-28")?;

    if let Ok(token) = ctx.secret("GITHUB_TOKEN") {
        headers.set("Authorization", &format!("Bearer {token}"))?;
    }

    fetch_text(url, headers).await
}

async fn fetch_public_asset_text(url: &str) -> Result<String> {
    let headers = Headers::new();
    headers.set("Accept", "application/json")?;
    headers.set("User-Agent", "worth-releases-worker")?;

    fetch_text(url, headers).await
}

async fn fetch_text(url: &str, headers: Headers) -> Result<String> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);

    let request = Request::new_with_init(url, &init)?;
    let mut response = Fetch::Request(request).send().await?;
    let status_code = response.status_code();

    if !(200..=299).contains(&status_code) {
        return Err(Error::RustError(format!(
            "GitHub returned HTTP {status_code} for {url}"
        )));
    }

    response.text().await
}

fn json_response<T: Serialize>(value: &T, max_age_seconds: Option<u32>) -> Result<Response> {
    let mut response = Response::from_json(value)?;
    set_json_headers(&mut response, max_age_seconds)?;
    Ok(response)
}

fn json_text_response(text: String, max_age_seconds: u32) -> Result<Response> {
    let mut response = Response::ok(text)?;
    set_json_headers(&mut response, Some(max_age_seconds))?;
    Ok(response)
}

fn redirect_response(url: Url, max_age_seconds: u32) -> Result<Response> {
    let mut response = Response::redirect(url)?;
    response.headers_mut().set(
        "Cache-Control",
        &format!("public, max-age={max_age_seconds}"),
    )?;
    response
        .headers_mut()
        .set("Access-Control-Allow-Origin", "*")?;
    Ok(response)
}

fn text_error(message: impl Into<String>, status: u16) -> Result<Response> {
    let mut response = Response::error(message.into(), status)?;
    response
        .headers_mut()
        .set("Access-Control-Allow-Origin", "*")?;
    response.headers_mut().set("Cache-Control", "no-store")?;
    Ok(response)
}

fn upstream_unavailable(error: &Error) -> Result<Response> {
    console_error!("Upstream request failed: {error}");
    text_error("No healthy upstream", 502)
}

fn set_json_headers(response: &mut Response, max_age_seconds: Option<u32>) -> Result<()> {
    response
        .headers_mut()
        .set("Content-Type", "application/json; charset=utf-8")?;
    response
        .headers_mut()
        .set("Access-Control-Allow-Origin", "*")?;
    response
        .headers_mut()
        .set("Cache-Control", &cache_control(max_age_seconds))?;
    Ok(())
}

fn cache_control(max_age_seconds: Option<u32>) -> String {
    max_age_seconds.map_or_else(
        || "no-store".to_string(),
        |max_age| {
            if max_age == 0 {
                "no-store".to_string()
            } else {
                format!("public, max-age={max_age}")
            }
        },
    )
}

fn releases_base_url(req: &Request) -> Result<Url> {
    let mut url = req.url()?;
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn cache_url(base_url: &Url, path: &str) -> Result<String> {
    let mut url = base_url.clone();
    url.set_path(path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(String::from(url))
}

fn stable_platform_download_url(base_url: &Url, platform: &str) -> Result<String> {
    let mut url = base_url.clone();
    url.path_segments_mut()
        .map_err(|()| Error::RustError("Unable to build stable download URL".to_string()))?
        .clear()
        .extend(["v1", "download", "stable", platform]);
    Ok(String::from(url))
}

fn display_version(tag_name: &str) -> &str {
    tag_name.strip_prefix('v').unwrap_or(tag_name)
}

fn find_stable_asset<'a>(release: &'a GitHubRelease, platform: &str) -> Option<&'a GitHubAsset> {
    match platform {
        "windows" => find_asset(release, |name| name.ends_with(".exe")),
        "macos-aarch64" => find_asset(release, |name| {
            name.ends_with(".dmg") && contains_any(name, &["aarch64", "arm64", "apple", "silicon"])
        }),
        "macos-x86_64" => find_asset(release, |name| {
            name.ends_with(".dmg") && contains_any(name, &["x86_64", "x64", "amd64", "intel"])
        }),
        "linux" => find_asset(release, |name| name.ends_with(".appimage")),
        _ => None,
    }
}

fn find_asset<F>(release: &GitHubRelease, mut matches: F) -> Option<&GitHubAsset>
where
    F: FnMut(&str) -> bool,
{
    release.assets.iter().find(|asset| {
        let name = asset.name.to_ascii_lowercase();
        !name.ends_with(".sig") && matches(&name)
    })
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn rewrite_update_manifest(text: &str, base_url: &Url) -> Result<String> {
    let mut manifest: serde_json::Value = serde_json::from_str(text)?;

    if let Some(platforms) = manifest
        .get_mut("platforms")
        .and_then(serde_json::Value::as_object_mut)
    {
        for platform in platforms.values_mut() {
            if let Some(url) = platform
                .get("url")
                .and_then(serde_json::Value::as_str)
                .and_then(|url| rewrite_update_download_url(url, base_url))
            {
                platform["url"] = serde_json::Value::String(url);
            }
        }
    }

    serde_json::to_string(&manifest).map_err(Error::from)
}

fn rewrite_update_download_url(url: &str, base_url: &Url) -> Option<String> {
    let url = Url::parse(url).ok()?;

    if url.host_str() != Some("github.com") {
        return None;
    }

    let segments = url.path_segments()?.collect::<Vec<_>>();

    if segments.len() != 6
        || !segments[0].eq_ignore_ascii_case(GITHUB_OWNER)
        || !segments[1].eq_ignore_ascii_case(GITHUB_REPO)
        || segments[2] != "releases"
        || segments[3] != "download"
    {
        return None;
    }

    versioned_download_url(base_url.clone(), segments[4], segments[5])
        .ok()
        .map(String::from)
}

fn validate_version(version: &str) -> bool {
    let normalized = version.strip_prefix('v').unwrap_or(version);

    !version.is_empty()
        && !normalized.is_empty()
        && version
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        && normalized
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit())
}

fn validate_filename(filename: &str) -> bool {
    !filename.is_empty()
        && !filename.contains('/')
        && !filename.contains('\\')
        && !filename.contains("..")
        && !filename.chars().any(char::is_control)
}

fn versioned_download_url(mut base_url: Url, version: &str, filename: &str) -> Result<Url> {
    base_url.set_query(None);
    base_url.set_fragment(None);
    base_url
        .path_segments_mut()
        .map_err(|()| Error::RustError("Unable to build versioned download URL".to_string()))?
        .clear()
        .extend(["v1", "download", version, filename]);
    Ok(base_url)
}

fn github_download_url(version: &str, filename: &str) -> Result<Url> {
    let mut url = Url::parse("https://github.com")?;
    url.path_segments_mut()
        .map_err(|()| Error::RustError("Unable to build GitHub download URL".to_string()))?
        .extend([
            GITHUB_OWNER,
            GITHUB_REPO,
            "releases",
            "download",
            version,
            filename,
        ]);
    Ok(url)
}

fn decoded_route_param(ctx: &RouteContext<()>, name: &str) -> Result<String> {
    let value = ctx
        .param(name)
        .ok_or_else(|| Error::RustError(format!("Missing `{name}` route parameter")))?;

    js_sys::decode_uri_component(value)
        .map_err(Error::from)?
        .as_string()
        .ok_or_else(|| Error::RustError(format!("Unable to decode `{name}` route parameter")))
}
