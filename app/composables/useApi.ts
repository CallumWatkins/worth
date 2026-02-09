import type { ApiError, Result } from "~/bindings";
import { commands } from "~/bindings";

export const formatApiError = (error: ApiError): string => {
  if (error === "Db") return "Database error";
  if (error === "NotFound") return "Not found";
  if (typeof error === "object" && error && "Validation" in error)
    return `Validation error: ${error.Validation}`;

  return "Unknown error";
};

export class ApiCommandError extends Error {
  public readonly apiError: ApiError;

  constructor(apiError: ApiError) {
    super(formatApiError(apiError));
    this.name = "ApiCommandError";
    this.apiError = apiError;
  }
}

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
