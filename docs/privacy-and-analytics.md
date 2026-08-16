# Privacy and analytics

The rules for analytics, error reporting, surveys, network requests, and user data.

Worth is local-first. Financial records and imported data belong on the user's device. Product analytics and feedback are optional and must remain useful without describing the user's finances.

## Data boundary

Never send account or institution names, balances, snapshot values or dates, currencies tied to an account, search text, resource IDs, filenames, imported file contents, or raw database/validation errors.

Safe analytics properties are generally counts, booleans, selected option indexes, operation durations, app versions, and coarse error categories. Backend validation issues may include a curated `telemetry_message`; use that value rather than the user-facing message, and omit it when no safe generalization exists.

User-written survey responses are sent only when the user deliberately submits them. Do not attach hidden financial context to survey events.

## Collection paths

- Manual product events go through `useAnalytics`; autocapture, page views, session recording, heatmaps, and external dependency loading are disabled.
- Handled exceptions go through `app/utils/error-reporting.ts`.
- Query and mutation errors redact parameter values through `getRedactedQueryKey`.
- The settings plugin applies the persisted analytics opt-in/out choice to PostHog.
- Production builds may upload frontend source maps to PostHog during packaging, then remove them from packaged assets. This is build-time behavior, not user-data collection.

The other runtime network integration is the production updater at `releases.useworth.app`. External project/support links open only after user action and are allowlisted in Tauri capabilities.

## Adding network behavior

Before adding a request or analytics property:

1. Confirm it cannot contain financial or identifying user content.
2. Use the existing analytics/error helpers so opt-out and redaction apply.
3. Update both development CSP in `nuxt.config.ts` and packaged CSP in `src-tauri/tauri.conf.json` when a new connection origin is required.
4. Update Tauri capabilities for newly opened external URLs.
5. Update this document and user-facing privacy copy when the data boundary changes.

Prefer collecting less. If a property is merely convenient rather than necessary for a concrete product question, leave it out.
