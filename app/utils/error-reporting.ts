import type { Properties } from "posthog-js";
import type { ApiError } from "~/generated/bindings";
import posthog from "posthog-js";
import { getRedactedQueryKey } from "~/utils/query";

const reportedErrorObjects = new WeakSet<object>();

export function reportHandledError(error: unknown, properties: Properties = {}) {
  if (!import.meta.client || !posthog.__loaded) return;
  if (typeof error === "object" && error !== null) {
    if (reportedErrorObjects.has(error)) return;
    reportedErrorObjects.add(error);
  }

  try {
    posthog.captureException(error, {
      handled_by_app: true,
      ...properties
    });
  } catch { }
}

export function getApiErrorKind(error: ApiError): "Db" | "NotFound" | "Validation" | "Unknown" {
  if (error === "Db" || error === "NotFound") return error;
  if (typeof error === "object" && "Validation" in error) return "Validation";
  return "Unknown";
}

export function getApiErrorProperties(error: ApiError): Properties {
  if (typeof error === "object" && "Validation" in error) {
    return {
      api_error_kind: "Validation",
      validation_issue_count: error.Validation.length,
      validation_fields: error.Validation.map((issue) => issue.field),
      validation_messages: error.Validation.map((issue) => issue.telemetry_message)
    };
  }

  return { api_error_kind: getApiErrorKind(error) };
}

export function createReportedApiError(error: ApiError) {
  const reportedError = new Error(`API command failed: ${getApiErrorKind(error)}`);
  reportedError.name = "ApiCommandError";
  return reportedError;
}

export function getQueryKeyProperties(queryKey: readonly unknown[] | undefined): Properties {
  if (!queryKey) return {};

  return {
    query_key: getRedactedQueryKey(queryKey),
    query_key_length: queryKey.length
  };
}
