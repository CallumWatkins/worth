import type { ApiError, Result, ValidationIssue } from "~/generated/bindings";
import { commands } from "~/generated/bindings";

export const formatApiError = (error: ApiError): string => {
  if (error === "Db") return "Database error";
  if (error === "NotFound") return "Not found";
  if (typeof error === "object" && error && "Validation" in error) {
    const joined = error.Validation.map((issue) => issue.message).join("; ");
    return joined.length ? `Validation error: ${joined}` : "Validation error";
  }

  return "Unknown error";
};

export const validationIssuesFromApiError = (error: ApiError): ValidationIssue[] => {
  if (typeof error === "object" && error && "Validation" in error) {
    return error.Validation;
  }
  return [];
};

export class ApiCommandError extends Error {
  public readonly apiError: ApiError;

  constructor(apiError: ApiError) {
    super(formatApiError(apiError));
    this.name = "ApiCommandError";
    this.apiError = apiError;
  }
}

export const extractValidationIssues = (error: unknown): ValidationIssue[] => {
  if (error instanceof ApiCommandError) {
    return validationIssuesFromApiError(error.apiError);
  }
  return [];
};

export const validationIssuesToFieldMap = (
  issues: ValidationIssue[]
): Record<string, string> => {
  const out: Record<string, string> = {};
  for (const issue of issues) {
    if (!out[issue.field]) {
      out[issue.field] = issue.message;
    }
  }
  return out;
};

export const extractValidationFieldErrors = (
  error: unknown
): Record<string, string> => {
  return validationIssuesToFieldMap(extractValidationIssues(error));
};

export const unwrapResult = <T>(result: Result<T, ApiError>): T => {
  if (result.status === "ok")
    return result.data;

  throw new ApiCommandError(result.error);
};

export const invokeResult = async <T>(promise: Promise<Result<T, ApiError>>): Promise<T> => {
  return unwrapResult(await promise);
};

type UnwrapCommand<F>
  = F extends (...args: infer A) => Promise<Result<infer T, ApiError>>
    ? (...args: A) => Promise<T>
    : never;

export type Api = {
  [K in keyof typeof commands]: UnwrapCommand<(typeof commands)[K]>
};

const api: Api = new Proxy(commands as any, {
  get(target, prop) {
    const fn = target[prop];
    if (typeof fn !== "function") return fn;

    return (...args: any[]) => invokeResult(fn(...args));
  }
});

export const useApi = (): Api => {
  return api;
};
