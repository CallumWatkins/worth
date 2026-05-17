# Releases

Worth releases are created by pushing a signed stable version tag, such as `v1.2.3`.

## Requirements

- The tag must be a signed annotated tag verified by GitHub.
- The tagged commit must be contained in `master`.
- `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` must match the tag version without the leading `v`.

## Process

1. Bump the version with `bun run bump`.
2. Refresh `src-tauri/Cargo.lock` if needed by running `bun run check:rust`, then commit any lockfile change.
3. Merge the release commit into `master`.
4. Create a signed annotated tag with `git tag -s v1.2.3 -m v1.2.3`.
5. Push `master` and the tag with `git push origin master v1.2.3`.
6. Review the generated draft GitHub Release, confirm the assets and `latest.json`, then publish.
