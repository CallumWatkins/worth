# CI And Repository Rules

Worth uses GitHub Actions for pull request checks and GitHub repository rulesets for merge and release-tag protection.

## Required CI

Pull requests targeting `master` run `.github/workflows/ci.yml`. The only required status check in GitHub is the aggregate `ci-required` job from GitHub Actions.

The aggregate job depends on these checks:

| Job | Purpose |
| --- | --- |
| `changes` | Detect whether releases worker files changed. |
| `migrations` | Reject changes to migrations frozen by a stable release tag and test the checker. |
| `ts-lint` | Run strict TypeScript/Vue ESLint. |
| `ts-typecheck` | Run Nuxt typecheck. |
| `rust-fmt` | Check Rust app formatting. |
| `rust-check` | Run locked Rust app `cargo check`. |
| `rust-clippy` | Run locked Rust app Clippy with warnings denied. |
| `rust-test` | Run locked Rust app tests. |
| `versions` | Check source version sync. |
| `contracts` | Regenerate committed contracts and fail on drift. |
| `licenses` | Run release-target license policy checks. |
| `releases-worker-*` | Run releases worker Cargo checks and compiled-route tests when releases worker paths changed. |

Local app CI simulation:

```sh
bun install --frozen-lockfile
bun run check:ci
```

The local `check:ci` script covers the app and release gates. Releases worker checks are worker-local and are run by GitHub Actions when releases worker paths change.

## Released migrations

A migration becomes immutable when it is included in a stable `v*.*.*` tag. The migration check compares `HEAD` with the highest stable tag, rejecting changes, deletions, and renames of migration files present in that tag. Migrations added after the tag remain editable until the next stable tag is created.

The release workflow excludes the tag currently being validated and compares against the preceding stable tag. If a release attempt is abandoned, delete its tag before changing a migration it introduced.

Git normalizes repository text files to LF through `.gitattributes`. Migration files have an explicit LF rule because SQLx hashes their exact bytes. Rust tests execute the current migration chain against an empty SQLite database and upgrade a database built from the latest stable tag.

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
