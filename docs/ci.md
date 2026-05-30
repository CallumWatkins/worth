# CI And Repository Rules

Worth uses GitHub Actions for pull request checks and GitHub repository rulesets for merge and release-tag protection.

## Required CI

Pull requests targeting `master` run `.github/workflows/ci.yml`. The only required status check in GitHub is the aggregate `ci-required` job from GitHub Actions.

The aggregate job depends on these checks:

| Job | Purpose |
| --- | --- |
| `changes` | Detect whether releases worker files changed. |
| `ts-lint` | Run strict TypeScript/Vue ESLint. |
| `ts-typecheck` | Run Nuxt typecheck. |
| `rust-fmt` | Check Rust app formatting. |
| `rust-check` | Run locked Rust app `cargo check`. |
| `rust-clippy` | Run locked Rust app Clippy with warnings denied. |
| `rust-test` | Run locked Rust app tests. |
| `versions` | Check source version sync. |
| `contracts` | Regenerate committed contracts and fail on drift. |
| `licenses` | Run release-target license policy checks. |
| `releases-worker-*` | Run releases worker Cargo checks when releases worker paths changed. |

Local app CI simulation:

```sh
bun install --frozen-lockfile
bun run check:ci
```

The local `check:ci` script covers the app and release gates. Releases worker checks are worker-local and are run by GitHub Actions when releases worker paths change.

## `master` Ruleset

Ruleset name: `Protect master`.

Target:

| Setting | Value |
| --- | --- |
| Target type | Branch |
| Include | `~DEFAULT_BRANCH` |
| Enforcement | Active |

Bypass actors:

| Actor | Mode |
| --- | --- |
| Repository owner | Pull request bypass |

Rules:

| Rule | Setting |
| --- | --- |
| Require pull request | Enabled |
| Required approving reviews | `0` |
| Required status checks | `ci-required` |
| Status check source | GitHub Actions |
| Require branch up to date | Enabled |
| Block non-fast-forward updates | Enabled |

Merge authority is handled by repository permissions: only the repository owner should have merge permission.

## Release Tag Ruleset

Ruleset name: `Protect release tags`.

Target:

| Setting | Value |
| --- | --- |
| Target type | Tag |
| Include | `refs/tags/v*.*.*` |
| Enforcement | Active |

Bypass actors:

| Actor | Mode |
| --- | --- |
| Repository owner | Always |

Rules:

| Rule | Setting |
| --- | --- |
| Restrict creations | Enabled |
| Restrict updates | Enabled |
| Restrict deletions | Enabled |
| Block non-fast-forward updates | Enabled |

The tag ruleset protects release tag refs. It does not replace `.github/workflows/release.yml`, which still validates SemVer format, signed annotated tag verification, source version sync, tag containment in `master`, duplicate release state, and native package builds.
