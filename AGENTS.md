## Worth — agent guide

Worth is a balance tracking desktop app built with **Tauri 2 (Rust)** + **Nuxt 4 (Vue 3, TypeScript)**.

### App scope

- Designed to track a **daily balance** for each account, entered manually by the user
- A **missing day** means the balance is unchanged since the previous stored value
- All data is stored **locally only** (not saved in the cloud)

### Tech stack

- **Frontend**: Nuxt 4, Nuxt UI 4, TailwindCSS 4, TypeScript, TanStack Vue Query, ECharts
- **Desktop**: Tauri 2 (`@tauri-apps/api`, `@tauri-apps/cli`)
- **Backend**: Rust 2021, `tokio`, `sqlx` (SQLite)
- **Type sharing**: `specta` / `tauri-specta` generates `app/bindings.ts`

### Repo layout (high level)

- **`app/`**: Nuxt app (pages/components/composables/plugins)
- **`nuxt.config.ts`**: Nuxt config (SSR off; static `nuxt generate` output)
- **`src-tauri/`**: Tauri/Rust app + config
  - **`src-tauri/src/api/`**: Tauri commands + DTOs (`#[tauri::command]`)
  - **`src-tauri/src/db/`**: Rust DB helpers/queries
  - **`src-tauri/db/`**: SQL migrations + seed scripts
  - **`src-tauri/src/bin/db.rs`**: DB dev CLI (seed/backup/restore/clear)
  - **`src-tauri/src/bin/export_bindings.rs`**: forces TS bindings export

### Common commands

```sh
# install dependencies (repo enforces bun)
bun install

# desktop dev (Nuxt dev server + Tauri)
bun run tauri:dev

# build desktop bundle
bun run tauri:build

# typecheck
bun run check:<ts|rust|all>

# lint
bun run lint:<ts|rust|all>[:fix]
```

- **Prereqs**: install the OS toolchain per [Tauri prerequisites](https://tauri.app/start/prerequisites).
- **Node version**: `.nvmrc` (currently 24); Bun is the supported package manager (`package.json` preinstall guard).

### Database (SQLite)

- **Engine**: SQLite via `sqlx`.
- **DB file location (runtime)**: `AppLocalData/db/worth.sqlite` (on Windows this is under `%LOCALAPPDATA%/<bundle id>/db/`).
- **Migrations**: `src-tauri/db/migrations` (run on app startup in `src-tauri/src/lib.rs`).
- **Seed/backup/restore CLI** (operates on the same app-local DB, `bun run db --help`)

#### DB operations + models

- **Database queries**: `src-tauri/src/db/mod.rs`
- **Models returned from queries**: `src-tauri/src/db/mod.rs` (query/aggregate/join result structs)
- **Database table models**: `src-tauri/src/db/rows.rs` (every table must have an up-to-date `sqlx::FromRow` model here)
- **API response models (DTOs)**: `src-tauri/src/api/mod.rs` (types returned over IPC; map DB models to DTOs)

```sh
# export Tauri IPC bindings to app/bindings.ts
cargo run --manifest-path src-tauri/Cargo.toml --bin export_bindings
```

- **Frontend API wrapper**: `app/composables/useApi.ts` wraps commands from `bindings.ts` and automatically unwraps `Result<T, ApiError>` to `T`.
