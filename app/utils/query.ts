import type { BalanceOverTimePeriod } from "~/bindings";

export const queryKeys = {
  accounts: {
    prefixes: {
      root: () => ["accounts"] as const,
      account: (accountId: number) => ["accounts", accountId] as const
    },
    list: () => ["accounts", "list"] as const,
    get: (accountId: number) => ["accounts", accountId, "get"] as const,
    snapshots: (accountId: number) => ["accounts", accountId, "snapshots"] as const,
    balanceOverTime: (accountId: number, period: BalanceOverTimePeriod) => ["accounts", accountId, "balanceOverTime", period] as const
  },
  institutions: {
    prefixes: {
      root: () => ["institutions"] as const,
      institution: (institutionId: number) => ["institutions", institutionId] as const
    },
    list: () => ["institutions", "list"] as const,
    get: (institutionId: number) => ["institutions", institutionId, "get"] as const
  },
  dashboard: {
    prefixes: {
      root: () => ["dashboard"] as const
    },
    summary: () => ["dashboard", "summary"] as const,
    balanceOverTime: (period: BalanceOverTimePeriod) => ["dashboard", "balanceOverTime", period] as const
  },
  search: {
    prefixes: {
      root: () => ["search"] as const
    },
    get: (query: string) => ["search", "get", query] as const
  }
};
