use serde::{Deserialize, Serialize};
use worker::*;

const GITHUB_OWNER: &str = "CallumWatkins";
const GITHUB_REPO: &str = "worth";
const RELEASE_METADATA_CACHE_URL: &str =
    "https://releases.useworth.app/__cache/github-release-latest";
const RELEASE_BY_TAG_CACHE_PREFIX: &str =
    "https://releases.useworth.app/__cache/github-release-tag-";
const STABLE_JSON_CACHE_PATH: &str = "/__cache/stable-json";
const LATEST_JSON_CACHE_PATH: &str = "/__cache/latest-json";
const LATEST_JSON_TTL_SECONDS: u32 = 60;
const STABLE_JSON_TTL_SECONDS: u32 = 60;
const RELEASE_METADATA_TTL_SECONDS: u32 = 300;
const SPECIFIC_DOWNLOAD_REDIRECT_TTL_SECONDS: u32 = 3600;

const STABLE_SYSTEMS: &[StableSystemDescriptor] = &[
    StableSystemDescriptor {
        id: "windows",
        label: "Windows",
    },
    StableSystemDescriptor {
        id: "macos",
        label: "macOS",
    },
    StableSystemDescriptor {
        id: "linux",
        label: "Linux",
    },
];

const WEBSITE_DOWNLOADS: &[StableDownloadDescriptor] = &[
    StableDownloadDescriptor {
        id: "windows-x86_64-setup",
        system_id: "windows",
        label: "Windows x64 Setup",
        os: "windows",
        arch: "x86_64",
        supports: &["x86_64"],
        format: "exe",
        kind: StableDownloadKind::WindowsX64Setup,
    },
    StableDownloadDescriptor {
        id: "windows-aarch64-setup",
        system_id: "windows",
        label: "Windows Arm64 Setup",
        os: "windows",
        arch: "aarch64",
        supports: &["aarch64"],
        format: "exe",
        kind: StableDownloadKind::WindowsAarch64Setup,
    },
    StableDownloadDescriptor {
        id: "macos-universal-dmg",
        system_id: "macos",
        label: "macOS Universal DMG",
        os: "macos",
        arch: "universal",
        supports: &["x86_64", "aarch64"],
        format: "dmg",
        kind: StableDownloadKind::MacosUniversalDmg,
    },
    StableDownloadDescriptor {
        id: "linux-x86_64-appimage",
        system_id: "linux",
        label: "Linux x64 AppImage",
        os: "linux",
        arch: "x86_64",
        supports: &["x86_64"],
        format: "AppImage",
        kind: StableDownloadKind::LinuxX64AppImage,
    },
    StableDownloadDescriptor {
        id: "linux-x86_64-deb",
        system_id: "linux",
        label: "Linux x64 DEB",
        os: "linux",
        arch: "x86_64",
        supports: &["x86_64"],
        format: "deb",
        kind: StableDownloadKind::LinuxX64Deb,
    },
    StableDownloadDescriptor {
        id: "linux-aarch64-appimage",
        system_id: "linux",
        label: "Linux Arm64 AppImage",
        os: "linux",
        arch: "aarch64",
        supports: &["aarch64"],
        format: "AppImage",
        kind: StableDownloadKind::LinuxAarch64AppImage,
    },
    StableDownloadDescriptor {
        id: "linux-aarch64-deb",
        system_id: "linux",
        label: "Linux Arm64 DEB",
        os: "linux",
        arch: "aarch64",
        supports: &["aarch64"],
        format: "deb",
        kind: StableDownloadKind::LinuxAarch64Deb,
    },
];

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    id: u64,
    name: String,
    size: u64,
    digest: String,
    browser_download_url: String,
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
}

#[derive(Serialize)]
struct StableManifest {
    version: String,
    tag: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    systems: Vec<StableManifestSystem>,
}

#[derive(Serialize)]
struct StableManifestSystem {
    id: &'static str,
    label: &'static str,
    downloads: Vec<StableManifestDownload>,
}

#[derive(Serialize)]
struct StableManifestDownload {
    id: &'static str,
    label: &'static str,
    os: &'static str,
    arch: &'static str,
    supports: &'static [&'static str],
    format: &'static str,
    #[serde(rename = "fileName")]
    file_name: String,
    url: String,
    #[serde(rename = "sizeBytes")]
    size_bytes: u64,
    sha256: String,
}

struct StableSystemDescriptor {
    id: &'static str,
    label: &'static str,
}

struct StableDownloadDescriptor {
    id: &'static str,
    system_id: &'static str,
    label: &'static str,
    os: &'static str,
    arch: &'static str,
    supports: &'static [&'static str],
    format: &'static str,
    kind: StableDownloadKind,
}

#[derive(Clone, Copy)]
enum StableDownloadKind {
    WindowsX64Setup,
    WindowsAarch64Setup,
    MacosUniversalDmg,
    LinuxX64AppImage,
    LinuxX64Deb,
    LinuxAarch64AppImage,
    LinuxAarch64Deb,
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

async fn stable_update(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    cached_updater_manifest_response(&ctx, releases_base_url(&req)?)
        .await
        .or_else(|error| upstream_unavailable(&error))
}

async fn stable_manifest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let base_url = releases_base_url(&req)?;

    cached_stable_manifest_response(&ctx, base_url)
        .await
        .or_else(|error| upstream_unavailable(&error))
}

async fn cached_stable_manifest_response(
    ctx: &RouteContext<()>,
    base_url: Url,
) -> Result<Response> {
    let cache = Cache::default();
    let cache_key_url = cache_url(&base_url, STABLE_JSON_CACHE_PATH)?;
    let cache_key = Request::new(&cache_key_url, Method::Get)?;

    if let Some(response) = cache.get(&cache_key, false).await? {
        return Ok(response);
    }

    let release = latest_release(ctx).await?;
    let manifest = build_stable_manifest(&release, &base_url)?;
    let mut response = json_response(&manifest, Some(STABLE_JSON_TTL_SECONDS))?;

    cache.put(&cache_key, response.cloned()?).await?;

    Ok(response)
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

fn build_stable_manifest(release: &GitHubRelease, base_url: &Url) -> Result<StableManifest> {
    let systems = STABLE_SYSTEMS
        .iter()
        .map(|system| {
            let downloads = WEBSITE_DOWNLOADS
                .iter()
                .filter(|download| download.system_id == system.id)
                .filter_map(|download| {
                    find_stable_download_asset(release, download).map(|asset| {
                        let sha256 = asset
                            .digest
                            .strip_prefix("sha256:")
                            .ok_or_else(|| {
                                Error::RustError(format!(
                                    "Release asset `{}` does not have a sha256 digest",
                                    asset.name
                                ))
                            })?
                            .to_string();

                        Ok(StableManifestDownload {
                            id: download.id,
                            label: download.label,
                            os: download.os,
                            arch: download.arch,
                            supports: download.supports,
                            format: download.format,
                            file_name: asset.name.clone(),
                            url: String::from(versioned_download_url(
                                base_url.clone(),
                                &release.tag_name,
                                &asset.name,
                            )?),
                            size_bytes: asset.size,
                            sha256,
                        })
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(StableManifestSystem {
                id: system.id,
                label: system.label,
                downloads,
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|system| !system.downloads.is_empty())
        .collect::<Vec<_>>();

    if systems.is_empty() {
        return Err(Error::RustError(format!(
            "No installable release assets found for {}",
            release.tag_name
        )));
    }

    Ok(StableManifest {
        version: display_version(&release.tag_name).to_string(),
        tag: release.tag_name.clone(),
        published_at: release.published_at.clone(),
        systems,
    })
}

async fn cached_updater_manifest_response(
    ctx: &RouteContext<()>,
    base_url: Url,
) -> Result<Response> {
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
    let release_tag = update_manifest_release_tag(&text)?;
    let release_text = cached_github_api_text(
        ctx,
        &format!("{RELEASE_BY_TAG_CACHE_PREFIX}{release_tag}"),
        &format!(
            "https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/tags/{release_tag}"
        ),
        RELEASE_METADATA_TTL_SECONDS,
    )
    .await?;
    let release = serde_json::from_str(&release_text)?;
    let text = rewrite_update_manifest(&text, &base_url, &release)?;
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
    Ok(Response::builder()
        .with_status(302)
        .with_header("Location", url.as_str())?
        .with_header(
            "Cache-Control",
            &format!("public, max-age={max_age_seconds}"),
        )?
        .with_header("Access-Control-Allow-Origin", "*")?
        .empty())
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

fn display_version(tag_name: &str) -> &str {
    tag_name.strip_prefix('v').unwrap_or(tag_name)
}

fn find_stable_download_asset<'a>(
    release: &'a GitHubRelease,
    download: &StableDownloadDescriptor,
) -> Option<&'a GitHubAsset> {
    find_asset(release, |name| {
        stable_download_kind_matches(download.kind, name)
    })
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

fn stable_download_kind_matches(kind: StableDownloadKind, name: &str) -> bool {
    match kind {
        StableDownloadKind::WindowsX64Setup => {
            name.ends_with("-setup.exe") && contains_any(name, &["_x64-", "_x86_64-", "_amd64-"])
        }
        StableDownloadKind::WindowsAarch64Setup => {
            name.ends_with("-setup.exe") && contains_any(name, &["_arm64-", "_aarch64-"])
        }
        StableDownloadKind::MacosUniversalDmg => {
            name.ends_with(".dmg") && name.contains("universal")
        }
        StableDownloadKind::LinuxX64AppImage => {
            name.ends_with(".appimage") && contains_any(name, &["_amd64.", "_x64.", "_x86_64."])
        }
        StableDownloadKind::LinuxX64Deb => {
            name.ends_with(".deb") && contains_any(name, &["_amd64.", "_x64.", "_x86_64."])
        }
        StableDownloadKind::LinuxAarch64AppImage => {
            name.ends_with(".appimage") && contains_any(name, &["_aarch64.", "_arm64."])
        }
        StableDownloadKind::LinuxAarch64Deb => {
            name.ends_with(".deb") && contains_any(name, &["_aarch64.", "_arm64."])
        }
    }
}

fn rewrite_update_manifest(text: &str, base_url: &Url, release: &GitHubRelease) -> Result<String> {
    let mut manifest: serde_json::Value = serde_json::from_str(text)?;

    if let Some(platforms) = manifest
        .get_mut("platforms")
        .and_then(serde_json::Value::as_object_mut)
    {
        for platform in platforms.values_mut() {
            if let Some(url) = platform.get("url").and_then(serde_json::Value::as_str)
                && let Some(url) = rewrite_update_download_url(url, base_url, release)?
            {
                platform["url"] = serde_json::Value::String(url);
            }
        }
    }

    serde_json::to_string(&manifest).map_err(Error::from)
}

fn update_manifest_release_tag(text: &str) -> Result<String> {
    let manifest: serde_json::Value = serde_json::from_str(text)?;
    let version = manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::RustError("Updater manifest has no version".to_string()))?;
    let normalized = version.strip_prefix('v').unwrap_or(version);

    if !validate_version(normalized) {
        return Err(Error::RustError(
            "Updater manifest has an invalid version".to_string(),
        ));
    }

    Ok(format!("v{normalized}"))
}

fn rewrite_update_download_url(
    url: &str,
    base_url: &Url,
    release: &GitHubRelease,
) -> Result<Option<String>> {
    let Ok(url) = Url::parse(url) else {
        return Ok(None);
    };

    if url.host_str() == Some("github.com") {
        return Ok(github_release_download_parts(&url)
            .and_then(|(version, filename)| {
                versioned_download_url(base_url.clone(), version, filename).ok()
            })
            .map(String::from));
    }

    if url.host_str() != Some("api.github.com") {
        return Ok(None);
    }

    let Some(segments) = url.path_segments().map(Iterator::collect::<Vec<_>>) else {
        return Ok(None);
    };

    if segments.len() != 6
        || segments[0] != "repos"
        || !segments[1].eq_ignore_ascii_case(GITHUB_OWNER)
        || !segments[2].eq_ignore_ascii_case(GITHUB_REPO)
        || segments[3] != "releases"
        || segments[4] != "assets"
    {
        return Ok(None);
    }

    let asset_id = segments[5]
        .parse::<u64>()
        .map_err(|_| Error::RustError("Updater asset URL has an invalid asset ID".to_string()))?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .ok_or_else(|| {
            Error::RustError(format!(
                "Updater asset {asset_id} is not part of release {}",
                release.tag_name
            ))
        })?;
    let asset_url = Url::parse(&asset.browser_download_url)?;
    let (version, filename) = github_release_download_parts(&asset_url).ok_or_else(|| {
        Error::RustError(format!(
            "Updater asset `{}` does not have a Worth GitHub download URL",
            asset.name
        ))
    })?;

    if version != release.tag_name {
        return Err(Error::RustError(format!(
            "Updater asset `{}` belongs to release {version}, not {}",
            asset.name, release.tag_name
        )));
    }

    Ok(Some(String::from(versioned_download_url(
        base_url.clone(),
        version,
        filename,
    )?)))
}

fn github_release_download_parts(url: &Url) -> Option<(&str, &str)> {
    if url.host_str() != Some("github.com") {
        return None;
    }

    let segments = url.path_segments()?.collect::<Vec<_>>();

    (segments.len() == 6
        && segments[0].eq_ignore_ascii_case(GITHUB_OWNER)
        && segments[1].eq_ignore_ascii_case(GITHUB_REPO)
        && segments[2] == "releases"
        && segments[3] == "download")
        .then_some((segments[4], segments[5]))
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

#[cfg(test)]
mod tests {
    use super::{
        GitHubAsset, GitHubRelease, rewrite_update_download_url, rewrite_update_manifest,
        update_manifest_release_tag,
    };
    use worker::Url;

    fn release() -> GitHubRelease {
        GitHubRelease {
            tag_name: "v1.2.3".to_string(),
            published_at: "2026-08-22T00:00:00Z".to_string(),
            assets: vec![GitHubAsset {
                id: 12345,
                name: "Worth_1.2.3_x64-setup.nsis.zip".to_string(),
                size: 42,
                digest: "sha256:abc".to_string(),
                browser_download_url: "https://github.com/CallumWatkins/worth/releases/download/v1.2.3/Worth_1.2.3_x64-setup.nsis.zip".to_string(),
            }],
        }
    }

    #[test]
    fn rewrites_legacy_browser_download_urls() {
        let base_url = Url::parse("https://releases.useworth.app").unwrap();
        let rewritten = rewrite_update_download_url(
            "https://github.com/CallumWatkins/worth/releases/download/v1.2.3/Worth_1.2.3_x64-setup.nsis.zip",
            &base_url,
            &release(),
        )
        .unwrap();

        assert_eq!(
            rewritten.as_deref(),
            Some("https://releases.useworth.app/v1/download/v1.2.3/Worth_1.2.3_x64-setup.nsis.zip")
        );
    }

    #[test]
    fn rewrites_tauri_action_v1_asset_urls_using_release_metadata() {
        let base_url = Url::parse("https://releases.useworth.app").unwrap();
        let rewritten = rewrite_update_download_url(
            "https://api.github.com/repos/CallumWatkins/worth/releases/assets/12345",
            &base_url,
            &release(),
        )
        .unwrap();

        assert_eq!(
            rewritten.as_deref(),
            Some("https://releases.useworth.app/v1/download/v1.2.3/Worth_1.2.3_x64-setup.nsis.zip")
        );
    }

    #[test]
    fn rejects_unknown_tauri_action_v1_asset_ids() {
        let base_url = Url::parse("https://releases.useworth.app").unwrap();
        let result = rewrite_update_download_url(
            "https://api.github.com/repos/CallumWatkins/worth/releases/assets/67890",
            &base_url,
            &release(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_asset_metadata_from_a_different_release() {
        let base_url = Url::parse("https://releases.useworth.app").unwrap();
        let mut release = release();
        release.assets[0].browser_download_url = "https://github.com/CallumWatkins/worth/releases/download/v1.2.2/Worth_1.2.3_x64-setup.nsis.zip".to_string();
        let result = rewrite_update_download_url(
            "https://api.github.com/repos/CallumWatkins/worth/releases/assets/12345",
            &base_url,
            &release,
        );

        assert!(result.is_err());
    }

    #[test]
    fn derives_the_exact_release_tag_from_the_updater_manifest() {
        assert_eq!(
            update_manifest_release_tag(r#"{"version":"1.2.3"}"#).unwrap(),
            "v1.2.3"
        );
        assert_eq!(
            update_manifest_release_tag(r#"{"version":"v1.2.3"}"#).unwrap(),
            "v1.2.3"
        );
        assert!(update_manifest_release_tag(r#"{"version":"../latest"}"#).is_err());
    }

    #[test]
    fn rewrites_mixed_updater_manifests() {
        let base_url = Url::parse("https://releases.useworth.app").unwrap();
        let manifest = r#"{
            "version": "1.2.3",
            "platforms": {
                "windows-x86_64-nsis": {
                    "signature": "signature",
                    "url": "https://api.github.com/repos/CallumWatkins/worth/releases/assets/12345"
                },
                "linux-x86_64-appimage": {
                    "signature": "signature",
                    "url": "https://github.com/CallumWatkins/worth/releases/download/v1.2.3/Worth_1.2.3_x64-setup.nsis.zip"
                }
            }
        }"#;

        let rewritten = rewrite_update_manifest(manifest, &base_url, &release()).unwrap();

        assert!(!rewritten.contains("api.github.com"));
        assert!(!rewritten.contains("github.com/CallumWatkins/worth/releases/download"));
        assert_eq!(
            rewritten
                .matches("releases.useworth.app/v1/download")
                .count(),
            2
        );
    }
}
