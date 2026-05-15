# Worth Releases Worker

Cloudflare Worker for `https://releases.useworth.app`.

## Routes

- `GET /health`
- `GET /v1/update/stable/:target/:arch/:currentVersion`
- `GET /v1/stable.json`
- `GET|HEAD /v1/download/stable/:platform`
- `GET|HEAD /v1/download/:version/:filename`

Stable download platform IDs are `windows`, `macos-aarch64`, `macos-x86_64`, and `linux`.

## Local Development

```sh
bunx wrangler dev --cwd workers/releases
```

## Deployment

Deployment is handled by the `deploy-releases-worker` GitHub Actions workflow when changes are merged into the master branch.

Required Cloudflare API token (`CLOUDFLARE_API_TOKEN`) permissions:
- Account / Workers Scripts Write
- Account / Account Settings Read
- Zone / Workers Routes Write

Required GitHub token (`RELEASES_WORKER_GITHUB_TOKEN`) permissions:
- Repository / Contents Read
