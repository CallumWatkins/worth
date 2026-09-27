import type { MaybeRefOrGetter } from "vue";
import type { AccountTypeName, InstitutionSummaryDto } from "~/generated/bindings";

export type InstitutionAccountState = "has-active" | "has-empty" | "only-active" | "only-empty" | "none";

export interface InstitutionFilters {
  types: AccountTypeName[]
  accounts: InstitutionAccountState[]
  balanceRange: BalanceRange
}

export function useInstitutionFilters(institutions: MaybeRefOrGetter<InstitutionSummaryDto[]>, filters: Ref<InstitutionFilters>) {
  const typeItems = computed(() => [...new Set(toValue(institutions).flatMap((institution) => institution.account_types))]
    .map((type) => ({
      label: ACCOUNT_TYPE_META[type].label,
      value: type,
      chip: { ui: { base: ACCOUNT_TYPE_META[type].chipClass } }
    }))
    .toSorted((a, b) => a.label.localeCompare(b.label)));

  const accountItems: { label: string, value: InstitutionAccountState }[][] = [
    [
      { label: "Has active accounts", value: "has-active" },
      { label: "Has empty accounts", value: "has-empty" },
      { label: "Only active accounts", value: "only-active" },
      { label: "Only empty accounts", value: "only-empty" }
    ],
    [{ label: "No accounts", value: "none" }]
  ];

  const hasFilters = computed(() => filters.value.types.length > 0 || filters.value.accounts.length > 0
    || filters.value.balanceRange.minimum !== null || filters.value.balanceRange.maximum !== null);

  const filteredInstitutions = computed(() => toValue(institutions).filter((institution) => {
    const { account_count: count, empty_account_count: empty } = institution;
    const accountStates: Record<InstitutionAccountState, boolean> = {
      "has-active": count > empty,
      "has-empty": empty > 0,
      "only-active": count > 0 && empty === 0,
      "only-empty": count > 0 && empty === count,
      none: count === 0
    };

    return (filters.value.types.length === 0 || filters.value.types.some((type) => institution.account_types.includes(type)))
      && (filters.value.accounts.length === 0 || filters.value.accounts.some((state) => accountStates[state]))
      && matchesBalanceRange(convertCurrencyMinorUnitsToMajorAmount(institution.total_balance_minor), filters.value.balanceRange);
  }));

  function resetFilters() {
    filters.value = { types: [], accounts: [], balanceRange: createBalanceRange() };
  }

  return { typeItems, accountItems, hasFilters, filteredInstitutions, resetFilters };
}
