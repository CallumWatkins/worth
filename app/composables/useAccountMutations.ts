import type { InvalidateOptions } from "@tanstack/vue-query";
import type { AccountUpsertInput } from "~/generated/bindings";

import { useIsMutating, useMutation, useQueryClient } from "@tanstack/vue-query";

export const useAccountMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();
  const pendingLabelWrites = useIsMutating({ mutationKey: ["account-labels"] });

  const invalidateAccountWrites = async (options: InvalidateOptions = {}) => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.labels.list() }, options),
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }, options),
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }, options),
      queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }, options),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() }, options)
    ]);
  };

  const createAccount = proxyRefs(useMutation({
    mutationFn: async (input: AccountUpsertInput) => api.accountsCreate(input),
    onSuccess: async () => invalidateAccountWrites()
  }));

  const updateAccount = proxyRefs(useMutation({
    mutationFn: async ({ accountId, input }: { accountId: number, input: AccountUpsertInput }) =>
      api.accountsUpdate(accountId, input),
    onSuccess: async () => invalidateAccountWrites()
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

  const setLabel = proxyRefs(useMutation({
    mutationKey: ["account-labels"],
    scope: { id: "account-labels" },
    mutationFn: async ({ accountId, labelId, assigned }: { accountId: number, labelId: number, assigned: boolean, invalidate: boolean }) =>
      api.accountsSetLabel(accountId, labelId, assigned),
    onSuccess: async (_, { accountId, invalidate }) => {
      // Discard pre-save reads even while the menu defers the account list refresh.
      const queryKey = queryKeys.accounts.get(accountId);
      await queryClient.cancelQueries({ queryKey });
      if (invalidate) await invalidateAccountWrites();
      else await queryClient.invalidateQueries({ queryKey });
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
    setLabel,
    pendingLabelWrites,
    deleteAccount,
    invalidateAccountWrites
  };
};
