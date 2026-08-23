# Worth Releases Worker

Cloudflare Worker for `https://releases.useworth.app`.

## Routes

- `GET /health`
- `GET /v1/update/stable/:target/:arch/:currentVersion`
- `GET /v1/stable.json`
- `GET|HEAD /v1/download/:version/:filename`

## Local Development

```sh
bunx wrangler dev --cwd workers/releases
```

## Tests

```sh
bun run test:releases-worker
```

The command performs a Wrangler deployment dry run, then exercises the compiled
Wasm worker through Wrangler's local test harness. Rust unit tests and the
separate Wasm check and Clippy gates remain part of CI.

## Deployment

Deployment is handled by the `deploy-releases-worker` GitHub Actions workflow when changes are merged into the master branch.

Required Cloudflare API token (`CLOUDFLARE_API_TOKEN`) permissions:
- Account / Workers Scripts Write
- Account / Account Settings Read
- Zone / Workers Routes Write

Required GitHub token (`RELEASES_WORKER_GITHUB_TOKEN`) permissions:
- Repository / Contents Read
