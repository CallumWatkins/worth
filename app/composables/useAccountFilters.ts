import type { MaybeRefOrGetter } from "vue";
import type { AccountDto, AccountTypeName } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

export interface AccountFilters {
  institutionIds: number[]
  labels: (number | "none")[]
  types: AccountTypeName[]
  statuses: ("open" | "closed")[]
  balances: ("active" | "empty")[]
  balanceRange: BalanceRange
}

export function useAccountFilters(accounts: MaybeRefOrGetter<AccountDto[]>, filters: Ref<AccountFilters>) {
  const api = useApi();
  const labelsQuery = useQuery({ queryKey: queryKeys.labels.list(), queryFn: api.labelsList });
  const labelItems = computed<{ label: string, value: number | "none" }[][]>(() => [
    (labelsQuery.data.value ?? [])
      .toSorted((a, b) => a.name.localeCompare(b.name))
      .map((label) => ({ label: label.name, value: label.id })),
    [{ label: "No labels", value: "none" }]
  ]);
  const institutionItems = computed(() => [...new Map(
    toValue(accounts).map((account) => [account.institution.id, {
      label: account.institution.name,
      value: account.institution.id
    }])
  ).values()].toSorted((a, b) => a.label.localeCompare(b.label)));

  const typeItems = computed(() => [...new Set(toValue(accounts).map((account) => account.account_type.name))]
    .map((type) => ({
      label: ACCOUNT_TYPE_META[type].label,
      value: type,
      chip: { ui: { base: ACCOUNT_TYPE_META[type].chipClass } }
    }))
    .toSorted((a, b) => a.label.localeCompare(b.label)));

  const statusItems = [
    { label: "Open", value: "open" as const },
    { label: "Closed", value: "closed" as const }
  ];
  const balanceItems = [
    { label: "Active", value: "active" as const },
    { label: "Empty", value: "empty" as const }
  ];

  const hasFilters = computed(() => (
    filters.value.institutionIds.length > 0 || filters.value.types.length > 0 || filters.value.labels.length > 0
    || filters.value.statuses.length > 0 || filters.value.balances.length > 0
    || filters.value.balanceRange.minimum !== null || filters.value.balanceRange.maximum !== null
  ));

  const filteredAccounts = computed(() => toValue(accounts).filter((account) => {
    const balance = convertCurrencyMinorUnitsToMajorAmount(account.latest_balance_minor);
    return (
      (filters.value.institutionIds.length === 0 || filters.value.institutionIds.includes(account.institution.id))
      && (filters.value.labels.length === 0
        || account.labels.some((label) => filters.value.labels.includes(label.id))
        || (filters.value.labels.includes("none") && account.labels.length === 0))
      && (filters.value.types.length === 0 || filters.value.types.includes(account.account_type.name))
      && (filters.value.statuses.length === 0 || filters.value.statuses.includes(account.closed_date == null ? "open" : "closed"))
      && (filters.value.balances.length === 0 || filters.value.balances.includes(account.latest_balance_minor === 0 ? "empty" : "active"))
      && matchesBalanceRange(balance, filters.value.balanceRange)
    );
  }));

  function resetFilters() {
    filters.value = { institutionIds: [], labels: [], types: [], statuses: [], balances: [], balanceRange: createBalanceRange() };
  }

  return { institutionItems, labelItems, typeItems, statusItems, balanceItems, hasFilters, filteredAccounts, resetFilters };
}
