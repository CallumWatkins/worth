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
    mutationFn: (input: AccountUpsertInput) => api.accountsCreate(input),
    onSuccess: invalidateAccountWrites
  }));

  const updateAccount = proxyRefs(useMutation({
    mutationFn: ({ accountId, input }: { accountId: number, input: AccountUpsertInput }) =>
      api.accountsUpdate(accountId, input),
    onSuccess: invalidateAccountWrites
  }));

  const deleteAccount = proxyRefs(useMutation({
    mutationFn: (accountId: number) => api.accountsDelete(accountId),
    onSuccess: invalidateAccountWrites
  }));

  return {
    createAccount,
    updateAccount,
    deleteAccount
  };
};
