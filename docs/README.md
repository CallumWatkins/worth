# Maintainer documentation

Setup instructions and common commands are in the root README and `AGENTS.md`.

## Architecture

- [Architecture overview](architecture/overview.md) — how Nuxt, Tauri, and SQLite fit together and divide responsibility.
- [Domain and balance semantics](architecture/domain-model.md) — accounts, snapshots, balances, dates, currencies, schema, and search.
- [Contracts and data access](architecture/contracts-and-data-access.md) — Rust-first IPC contracts, validation, generated types, queries, and cache invalidation.
- [Snapshot imports](architecture/snapshot-imports.md) — source inspection, preview planning, import policies, and transactional commits.
- [History navigation safety](architecture/history-navigation.md) — modal layers, route guards, deletion redirects, and browser Back/Forward handling.
- [Automatic updates](architecture/automatic-updates.md) — update checks, downloads, installation, platform behavior, and frontend state.

## Policies and delivery

- [Privacy and analytics](privacy-and-analytics.md) — local data boundaries, telemetry, error reporting, surveys, and network access.
- [CI and repository rules](ci.md) — workflows, required checks, branch protection, and repository rulesets.
- [Releases](releases.md) — versioning, signed tags, package builds, and publishing.
- [License policy](license-policy.md) — dependency license approvals, exceptions, overrides, and release checks.

Keep additions short. Prefer invariants and checklists over code walkthroughs, API catalogs, or details that generated types already express.
