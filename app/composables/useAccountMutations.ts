import type { AccountUpsertInput } from "~/generated/bindings";

import { useMutation, useQueryClient } from "@tanstack/vue-query";

export const useAccountMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();

  const invalidateAccountWrites = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() })
    ]);
  };

  const createAccount = proxyRefs(useMutation({
    mutationFn: async (input: AccountUpsertInput) => api.accountsCreate(input),
    onSuccess: invalidateAccountWrites
  }));

  const updateAccount = proxyRefs(useMutation({
    mutationFn: async ({ accountId, input }: { accountId: number, input: AccountUpsertInput }) =>
      api.accountsUpdate(accountId, input),
    onSuccess: invalidateAccountWrites
  }));

  const setDashboardInclusion = proxyRefs(useMutation({
    mutationFn: async ({ accountId, include }: { accountId: number, include: boolean }) =>
      api.accountsSetDashboardInclusion(accountId, include),
    onSuccess: async (_, { accountId }) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.accounts.list() }),
        queryClient.invalidateQueries({ queryKey: queryKeys.accounts.get(accountId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
        queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() })
      ]);
    }
  }));

  const deleteAccount = proxyRefs(useMutation({
    mutationFn: async ({ accountId }: { accountId: number, invalidate: boolean }) => api.accountsDelete(accountId),
    onSuccess: async (_, { invalidate }) => {
      if (invalidate) await invalidateAccountWrites();
    }
  }));

  return {
    createAccount,
    updateAccount,
    setDashboardInclusion,
    deleteAccount,
    invalidateAccountWrites
  };
};
