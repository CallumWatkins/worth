import type { BalanceOverTimePeriod } from "~/bindings";

import { commands } from "~/bindings";
import { invokeResult } from "~/utils/tauri-result";

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
