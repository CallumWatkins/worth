# Snapshot imports

How imported files are inspected, planned, previewed, and committed.

Snapshot imports deliberately separate source parsing from domain planning:

- `src-tauri/src/imports/snapshots/csv.rs` inspects CSV input, guesses options, and converts rows into candidates.
- `src-tauri/src/imports/snapshots/mod.rs` applies source-independent duplicate, existing-date, unchanged-value, and account-date policies.
- `src-tauri/src/api/mod.rs` loads current account state, requests a plan, and commits its writes.
- `app/composables/useCsvSnapshotImportFlow.ts` owns the three-step UI and sends the original source plus selected options to Rust.

Imported file contents cross only the local WebView-to-Rust IPC boundary. Do not send filenames, contents, raw dates, or raw amounts to analytics.

## Inspect, preview, commit

1. **Inspect** reads the source and returns columns, sample rows, and option guesses.
2. **Preview** parses all candidates and plans actions against current snapshots without writing.
3. **Commit** repeats planning against the current database, rejects invalid rows or unconfirmed overwrites, then applies every write in one transaction.

Commit must not trust a previously returned preview: snapshots may have changed while the dialog was open.

## Planning rules

Candidates are planned in chronological order so each imported row can affect the previous balance used by later rows. Preview rows are then returned in original source order.

Each row becomes exactly one action: create, overwrite, skip existing, skip unchanged, skip duplicate, skip blank amount, or invalid. Invalid rows block the entire commit. Warnings—for example dates outside the account range or future dates—do not block it.

Overwriting an existing date requires both the overwrite policy and explicit confirmation. A new snapshot equal to the preceding effective balance may be excluded as redundant. An exact match on an already stored date is always skipped unchanged.

## Extending imports

When adding a source:

1. Add source-specific input, options, inspection, and candidate conversion types.
2. Extend the tagged source enums and dispatch without moving shared policies into the parser.
3. Reuse the planner and transactional commit path.
4. Add parsing tests plus planner tests for duplicates, ordering, unchanged values, invalid rows, warnings, and overwrite confirmation.
5. Regenerate bindings and implement the frontend flow using `SnapshotImportFlowDefinition`.
