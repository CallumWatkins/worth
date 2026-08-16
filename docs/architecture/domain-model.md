# Domain and balance semantics

The data model and rules behind accounts, snapshots, balances, dates, currencies, and search.

## Core records

- An **institution** groups accounts. Institution names are unique.
- An **account** belongs to one institution and has a type, currency, asset/liability classification, and optional opened/closed dates. Account names are unique within an institution.
- A **snapshot** is an account balance on one calendar date. Only one snapshot may exist per account and date.
- **App settings** use a singleton row with `id = 1`.

Deleting an institution cascades to its accounts and snapshots. Deleting an account cascades to its snapshots. Deletion UI must preview that impact and follow the redirect rules in `history-navigation.md` when the deleted resource is the current route.

## Balance rules

Balances are signed integer minor units, such as pennies. Liability balances are entered and stored as negative values; `account_classification` describes the account but does not negate values automatically.

Balance values are restricted to `±99,999,999,999,999` minor units. Specta casts Rust `i64` values to TypeScript `number`, so this limit must remain safely representable in JavaScript and aligned between the database, Rust validation, and frontend conversion helpers.

A missing date means the balance is unchanged since the previous snapshot. Series therefore forward-fill each account independently. Dates before its first snapshot remain unknown; aggregate series treat an unknown account as contributing zero until its first snapshot.

The latest stored snapshot determines an account's latest balance, including a future-dated snapshot. Balance-over-time charts stop at the user's local `today`, so a future snapshot may be latest without appearing on a chart.

## Dates and currencies

Snapshot and opened/closed dates are calendar dates (`YYYY-MM-DD`), not instants. Avoid UTC conversion when reading a user's selected date. Timestamps such as `created_at` are UTC instants.

Worth does not perform foreign-exchange conversion. Account views format values using the account currency, while cross-account totals sum stored minor-unit values and format the result using the default display currency. Do not imply converted totals without adding an explicit conversion model.

## Schema and search

The schema lives in `src-tauri/db/migrations`; matching `sqlx::FromRow` table models live in `src-tauri/src/db/rows.rs`. Migrations run on app startup. Add a new numbered migration when existing local databases must be upgraded; only rewrite an applied migration when intentionally dropping that compatibility.

Global search uses the denormalized `search_fts` table. SQLite triggers keep institution, account, and account-type text synchronized. A schema or write-path change that affects searchable text must preserve those triggers.

When changing these rules, update SQL constraints, Rust validation and calculations, generated contracts, frontend formatting, seed data, and focused Rust tests together.
