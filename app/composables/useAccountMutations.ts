import type { AccountUpsertInput } from "~/bindings";

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

  const createAccount = useMutation({
    mutationFn: (input: AccountUpsertInput) => api.accountsCreate(input),
    onSuccess: invalidateAccountWrites
  });

  const updateAccount = useMutation({
    mutationFn: ({ accountId, input }: { accountId: number, input: AccountUpsertInput }) =>
      api.accountsUpdate(accountId, input),
    onSuccess: invalidateAccountWrites
  });

  return {
    createAccount,
    updateAccount
  };
};
