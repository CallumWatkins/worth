# Architecture overview

How the Nuxt frontend, Tauri backend, and SQLite database fit together.

Worth is a static Nuxt client hosted in a Tauri WebView. Rust owns persistence and domain operations; Vue owns presentation and interaction state. They communicate through generated Tauri commands and a small number of explicit Tauri events.

## Runtime flow

1. `src-tauri/src/lib.rs` registers plugins, opens the app-local SQLite database, runs migrations, stores shared state, and starts update checks.
2. Vue code calls `useApi`, which wraps the generated commands in `app/generated/bindings.ts` and turns Rust `Result` values into returned data or `ApiCommandError`.
3. Pages and components fetch through TanStack Query. Mutation composables call the same API and invalidate all affected query families.
4. Rust commands in `src-tauri/src/api/mod.rs` validate inputs, coordinate transactions, call `src-tauri/src/db/mod.rs`, and map database rows into IPC DTOs.
5. SQLite is the source of persisted application state. Migrations run automatically at startup.

## Ownership

| Concern | Owner |
| --- | --- |
| Tables, SQL, transactions | Rust database layer |
| Domain validation and IPC DTOs | Rust API/contracts layers |
| Generated TypeScript types and Zod schemas | Rust-first contract pipeline |
| Backend-state caching and invalidation | TanStack Query composables |
| Forms, dialogs, navigation, formatting | Vue/Nuxt |
| Update download and installation state | Rust update manager |

Do not duplicate domain validation or aggregate calculations in Vue. Client validation exists for immediate feedback, but Rust remains authoritative.

## Storage and network boundary

Financial data is stored under Tauri's `AppLocalData/db/` directory. Account names, institution names, balances, snapshots, search text, and imported files must stay on the device.

The packaged app has two intentional network integrations: PostHog analytics/feedback and the release updater. User-opened project links are allowlisted separately. See `docs/privacy-and-analytics.md` before adding telemetry or network access.

## Important entry points

| Area | File |
| --- | --- |
| Desktop startup | `src-tauri/src/lib.rs` |
| Commands and DTOs | `src-tauri/src/api/mod.rs` |
| Shared contracts | `src-tauri/src/contracts/mod.rs` |
| Database access | `src-tauri/src/db/mod.rs` |
| Frontend API wrapper | `app/composables/useApi.ts` |
| Query keys | `app/utils/query.ts` |
| App shell | `app/app.vue` |

When a change crosses layers, update every owning layer in the same change and regenerate contracts rather than hand-writing boundary types.
