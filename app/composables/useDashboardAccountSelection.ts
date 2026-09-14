import { useQuery } from "@tanstack/vue-query";

export function useDashboardAccountSelection() {
  const api = useApi();
  const { setDashboardInclusion } = useAccountMutations();
  const accountsQuery = proxyRefs(useQuery({
    queryKey: queryKeys.accounts.list(),
    queryFn: api.accountsList
  }));
  const accounts = computed(() => accountsQuery.data ?? []);
  const pendingInclusions = ref(new Map<number, boolean>());
  const saveError = ref<string>();
  let isSaving = false;
  const errorMessage = computed(() => accountsQuery.error?.message ?? saveError.value);
  const includedIds = computed<number[]>((previous) => {
    const ids = accounts.value
      .filter((account) => pendingInclusions.value.get(account.id) ?? account.include_in_dashboard)
      .map((account) => account.id);
    // A refreshed query must not make the menu treat an unchanged selection as
    // a programmatic change and move its highlight (and scroll position).
    return previous?.length === ids.length && ids.every((id, index) => id === previous[index]) ? previous : ids;
  });

  async function showAllAccounts() {
    return onSelectionChange(accounts.value.map((account) => account.id));
  }

  async function onSelectionChange(ids: (number | undefined)[]) {
    saveError.value = undefined;
    for (const account of accounts.value) {
      const include = ids.includes(account.id);
      if (include !== (pendingInclusions.value.get(account.id) ?? account.include_in_dashboard)) {
        pendingInclusions.value.set(account.id, include);
      }
    }

    if (isSaving) return;
    isSaving = true;
    try {
      // Show selections immediately and save in order, including clicks made
      // while a previous change is still saving or refreshing the dashboard.
      while (pendingInclusions.value.size) {
        const next = pendingInclusions.value.entries().next();
        if (next.done) break;
        const [accountId, include] = next.value;
        try {
          await setDashboardInclusion.mutateAsync({ accountId, include });
        } catch (error) {
          saveError.value = error instanceof Error ? error.message : "Failed to update account";
        } finally {
          // A newer selection for this account must still be saved. Otherwise
          // use the refreshed server value (or revert to it after a failed save).
          if (pendingInclusions.value.get(accountId) === include) {
            pendingInclusions.value.delete(accountId);
          }
        }
      }
    } finally {
      isSaving = false;
    }
  }

  return {
    accountsQuery,
    accounts,
    includedIds,
    errorMessage,
    onSelectionChange,
    showAllAccounts
  };
}
