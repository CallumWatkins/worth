import type { ApiError, Result, ValidationIssue } from "~/generated/bindings";
import { commands } from "~/generated/bindings";

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

export const unwrapResult = <T>(result: Result<T, ApiError>): T => {
  if (result.status === "ok")
    return result.data;

  throw new ApiCommandError(result.error);
};

export const invokeResult = async <T>(promise: Promise<Result<T, ApiError>>): Promise<T> => {
  return unwrapResult(await promise);
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
    return async (...args: unknown[]) => invokeResult(command(...args));
  }
}) as unknown as Api;

export const useApi = () => api;
