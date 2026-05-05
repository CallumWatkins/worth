import type { ApiError, Result, ValidationIssue } from "~/generated/bindings";
import { commands } from "~/generated/bindings";
import { createReportedApiError, getApiErrorProperties, reportHandledError } from "~/utils/error-reporting";

export const formatApiError = (error: ApiError): string => {
  if (error === "Db") return "Database error";
  if (error === "NotFound") return "Not found";
  if (typeof error === "object" && "Validation" in error) {
    const joined = error.Validation.map((issue) => issue.message).join("; ");
    return joined.length ? `Validation error: ${joined}` : "Validation error";
  }

  return "Unknown error";
};

export const validationIssuesFromApiError = (error: ApiError): ValidationIssue[] => {
  if (typeof error === "object" && "Validation" in error) {
    return error.Validation;
  }
  return [];
};

export class ApiCommandError extends Error {
  public readonly apiError: ApiError;
  public readonly commandName?: string;

  constructor(apiError: ApiError, commandName?: string) {
    super(formatApiError(apiError));
    this.name = "ApiCommandError";
    this.apiError = apiError;
    this.commandName = commandName;
  }
}

export const extractValidationIssues = (error: unknown): ValidationIssue[] => {
  if (error instanceof ApiCommandError) {
    return validationIssuesFromApiError(error.apiError);
  }
  return [];
};

export const unwrapResult = <T>(result: Result<T, ApiError>, commandName?: string): T => {
  if (result.status === "ok")
    return result.data;

  reportHandledError(createReportedApiError(result.error), {
    source: "api_result",
    command_name: commandName ?? "unknown",
    ...getApiErrorProperties(result.error)
  });

  throw new ApiCommandError(result.error, commandName);
};

export const invokeResult = async <T>(promise: Promise<Result<T, ApiError>>, commandName?: string): Promise<T> => {
  return unwrapResult(await promise, commandName);
};

export type Api = {
  [K in keyof typeof commands]:
  (typeof commands)[K] extends (...args: infer A) => Promise<Result<infer T, ApiError>>
    ? (...args: A) => Promise<T>
    : never;
};

type ResultCommand = (...args: unknown[]) => Promise<Result<unknown, ApiError>>;

const api = new Proxy(commands, {
  get(target, prop) {
    if (!(prop in target)) return undefined;

    const command = target[prop as keyof typeof target] as ResultCommand;
    const commandName = String(prop);
    return async (...args: unknown[]) => {
      try {
        return await invokeResult(command(...args), commandName);
      } catch (error) {
        if (!(error instanceof ApiCommandError)) {
          reportHandledError(error, {
            source: "api_transport",
            command_name: commandName
          });
        }

        throw error;
      }
    };
  }
}) as unknown as Api;

export const useApi = () => api;
