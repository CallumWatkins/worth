/*
 * Worth uses PostHog for anonymous product analytics and diagnostics only.
 * Autocapture, page views, session recording, heatmaps, and similar broad
 * collection features are disabled globally. App code should use this composable
 * for deliberate, manual events tied to meaningful user actions. Opt-in/opt-out
 * is managed centrally through settings.
 *
 * Event names must follow PostHog's `category:object_action` convention:
 * - `category` is the product area or context that the event occurred in, for
 * example `accounts` or `snapshot_import`.
 * - `object` is the thing being interacted with, for example `account` or
 *   `account_create` (the account creation modal itself).
 * - `action` is a present-tense verb from the allowed list below, for example
 *   `create`, `update`, `delete`, or `fail`.
 *
 * Together, this creates event names such as `onboarding:account_create` or
 * `accounts:account_delete_fail`.
 *
 * Privacy is the most important constraint. Never send names, balances, search
 * text, or anything else that describes a user's finances. Prefer counts,
 * booleans, indexes, durations, and coarse error categories.
 * If a setting is stored as a PostHog person property via `$set`, prefix it
 * with `desktop_app_`. Backend validation errors may include `telemetry_message`;
 * those messages are intentionally curated to be safe for analytics.
 */

import type { Properties } from "posthog-js";
import { ApiCommandError } from "~/composables/useApi";
import { getApiErrorKind } from "~/utils/error-reporting";

export type AnalyticsEventCategory
  = | "app"
    | "account"
    | "account_settings"
    | "accounts"
    | "institution"
    | "institution_settings"
    | "institutions"
    | "onboarding"
    | "search"
    | "settings"
    | "snapshot_import";

export type AnalyticsEventObject
  = | "app"
    | "account"
    | "account_create"
    | "account_delete"
    | "account_update"
    | "csv_import"
    | "csv_preview"
    | "institution"
    | "institution_create"
    | "institution_delete"
    | "institution_update"
    | "onboarding_skip_button"
    | "onboarding_step"
    | "result"
    | "results"
    | "results_generate"
    | "setting"
    | "setting_update"
    | "snapshot"
    | "snapshot_create"
    | "snapshot_delete"
    | "snapshot_update";

export type AnalyticsEventVerb
  = | "click"
    | "submit"
    | "create"
    | "view"
    | "add"
    | "invite"
    | "update"
    | "delete"
    | "remove"
    | "start"
    | "end"
    | "cancel"
    | "fail"
    | "generate"
    | "send"
    | "install"
    | "upgrade"
    | "downgrade"
    | "open";

export type AnalyticsEventName = `${AnalyticsEventCategory}:${AnalyticsEventObject}_${AnalyticsEventVerb}`;

export type AnalyticsEventProperties = Properties & {
  operation_duration?: never
};

interface AnalyticsCaptureOptions {
  operationStartedAt?: number
}

export function useAnalytics() {
  const posthog = usePostHog();

  function captureAnalyticsEvent(eventName: AnalyticsEventName, properties: AnalyticsEventProperties = {}, options: AnalyticsCaptureOptions = {}) {
    const eventProperties: Properties = { ...properties };

    if (options.operationStartedAt != null) {
      eventProperties.operation_duration = Math.max(0, Math.round(performance.now() - options.operationStartedAt));
    }

    posthog?.capture(eventName, eventProperties);
  }

  return { captureAnalyticsEvent };
}

export function getAnalyticsErrorProperties(error: unknown): AnalyticsEventProperties {
  if (error instanceof ApiCommandError) {
    if (typeof error.apiError === "object" && "Validation" in error.apiError) {
      return {
        api_error_kind: "Validation",
        validation_issue_count: error.apiError.Validation.length,
        validation_fields: error.apiError.Validation.map((issue) => issue.field),
        validation_messages: error.apiError.Validation.map((issue) => issue.telemetry_message)
      };
    }

    return { api_error_kind: getApiErrorKind(error.apiError) };
  }

  if (error instanceof Error) return { error_name: error.name };

  return { error_name: typeof error };
}
