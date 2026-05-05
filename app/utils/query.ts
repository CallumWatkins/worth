import type { BalanceOverTimePeriod } from "~/generated/bindings";

interface QueryKeyParam<T> {
  readonly __paramName: string
  readonly value: T
}

const param = <T>(name: string, value: T): QueryKeyParam<T> => ({
  __paramName: name,
  value
});

export const getRedactedQueryKey = (queryKey: readonly unknown[]): unknown[] => {
  return queryKey.map((segment) => {
    if (typeof segment === "object" && segment != null && "__paramName" in segment && "value" in segment) {
      return `:${(segment as QueryKeyParam<unknown>).__paramName}`;
    }
    return segment;
  });
};

export const queryKeys = {
  settings: {
    get: () => ["settings", "get"] as const
  },
  accounts: {
    prefixes: {
      root: () => ["accounts"] as const,
      account: (accountId: number) => ["accounts", param("accountId", accountId)] as const
    },
    list: () => ["accounts", "list"] as const,
    get: (accountId: number) => ["accounts", param("accountId", accountId), "get"] as const,
    deletePreview: (accountId: number) => ["accounts", param("accountId", accountId), "deletePreview"] as const,
    snapshots: (accountId: number) => ["accounts", param("accountId", accountId), "snapshots"] as const,
    balanceOverTime: (accountId: number, period: BalanceOverTimePeriod) => ["accounts", param("accountId", accountId), "balanceOverTime", param("period", period)] as const
  },
  institutions: {
    prefixes: {
      root: () => ["institutions"] as const,
      institution: (institutionId: number) => ["institutions", param("institutionId", institutionId)] as const
    },
    list: () => ["institutions", "list"] as const,
    get: (institutionId: number) => ["institutions", param("institutionId", institutionId), "get"] as const,
    deletePreview: (institutionId: number) => ["institutions", param("institutionId", institutionId), "deletePreview"] as const
  },
  dashboard: {
    prefixes: {
      root: () => ["dashboard"] as const
    },
    summary: () => ["dashboard", "summary"] as const,
    balanceOverTime: (period: BalanceOverTimePeriod) => ["dashboard", "balanceOverTime", param("period", period)] as const
  },
  search: {
    prefixes: {
      root: () => ["search"] as const
    },
    get: (query: string) => ["search", "get", param("query", query)] as const
  }
};
