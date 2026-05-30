# Releases

Worth releases are created by pushing a signed stable version tag, such as `v1.2.3`.

## Requirements

- The tag must be a signed annotated tag verified by GitHub.
- The tagged commit must be contained in `master`.
- `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` must match the tag version without the leading `v`.
- The release commit must land through a pull request that passes the required `ci-required` check.
- Release tags matching `v*.*.*` are protected by a GitHub ruleset and can only be created, updated, or deleted by explicit bypass actors. See `docs/ci.md` for CI and repository ruleset settings.
- Worth release packages are built natively on each GitHub runner. Local cross-compilation is unsupported.

## Process

1. Bump the version with `bun run bump`.
2. Run `bun install --frozen-lockfile` and `bun run check:ci` to simulate the required app CI checks locally. See `docs/license-policy.md` before changing `license-policy.json`.
3. Open a pull request and merge the release commit into `master` after the required `ci-required` check passes.
4. Create a signed annotated tag with `git tag -s v1.2.3 -m ""`.
5. Push `master` and the tag with `git push origin master v1.2.3`.
6. Review the generated draft GitHub Release, confirm the assets and `latest.json`, then publish.
