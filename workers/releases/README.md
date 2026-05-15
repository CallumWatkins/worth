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
