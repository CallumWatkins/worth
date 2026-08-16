# Contracts and data access

How data moves between SQLite, Rust commands, generated contracts, and frontend queries.

## Layer boundaries

| Layer | Responsibility |
| --- | --- |
| `src-tauri/src/db/rows.rs` | One current `sqlx::FromRow` model for every table. |
| `src-tauri/src/db/mod.rs` | SQL, query result models, and transaction helpers. |
| `src-tauri/src/contracts/mod.rs` | Shared input types, enums, schemas, and validation metadata. |
| `src-tauri/src/api/mod.rs` | Tauri commands, authoritative validation, orchestration, and response DTOs. |
| `app/generated/` | Generated bindings, JSON Schemas, and Zod schemas. Never edit manually. |
| `app/composables/useApi.ts` | Typed command wrapper and API error conversion. |
| TanStack Query composables | Fetching, mutation state, cache updates, and invalidation. |

Database rows must not cross IPC directly. Commands map them to DTOs so storage details and frontend contracts can evolve independently.

## Rust-first contracts

Specta generates command signatures and TypeScript types. Types marked with `#[export_schema]` also produce JSON Schema; `json-schema-to-zod` converts those schemas for frontend forms.

For form inputs, keep all three representations aligned:

- `garde` performs authoritative Rust validation.
- `schemars` describes the constraint.
- `x-validation` supplies matching client-facing messages to the Zod generator.

`ApiError::Validation` paths use dot/index notation. Its `message` is shown to the user; `telemetry_message` must be generalized for analytics or omitted when no safe wording exists.

After adding or changing a command, include it in `specta_builder`, run `bun run contracts:gen`, and commit every generated change.

## Frontend server state

All application data fetching uses TanStack Query. Define reusable keys in `app/utils/query.ts`; wrap parameter values with `param(...)` so error telemetry records the key shape without resource IDs or search text.

Queries use infinite stale time. A successful mutation must therefore invalidate or update every affected cache. Keep that policy in the domain mutation composable, including list, detail, dashboard, institution, and search families as appropriate. Route-specific sequencing, such as delaying invalidation while redirecting from a deleted resource, stays with the caller.

## Change checklist

For a persisted field or table:

1. Update migrations and `db/rows.rs`.
2. Update database query/mutation models and SQL.
3. Update contracts, command validation, and DTO mapping.
4. Regenerate contracts and use the generated frontend types.
5. Update query keys/invalidation and add focused tests.

For a command without schema changes, start at step 3. Verify with `bun run contracts:gen`, `bun run check:all`, and the relevant lint/tests; `bun run check:ci` is the full local release gate.
