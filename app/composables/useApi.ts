import type { ApiError, BalanceOverTimePeriod, Result } from "~/bindings";
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

const api = {
  accounts: {
    list: () => invokeResult(commands.accountsList())
  },
  dashboard: {
    get: () => invokeResult(commands.dashboardGet()),
    balanceOverTime: (period: BalanceOverTimePeriod) =>
      invokeResult(commands.dashboardBalanceOverTime(period))
  }
};

export type Api = typeof api;

export const useApi = (): Api => {
  return api;
};
