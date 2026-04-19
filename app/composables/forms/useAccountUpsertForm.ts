import type { AccountDto, AccountTypeName, InstitutionSummaryDto } from "~/generated/bindings";
import { parseDate } from "@internationalized/date";

interface UseAccountUpsertFormParams {
  institutions: Ref<InstitutionSummaryDto[] | undefined>
  getDefaultInstitutionId?: () => number | null | undefined
}

function defaultInstitutionFallback(
  institutionItems: { value: { label: string, value: number | string }[] },
  getDefaultInstitutionId?: () => number | null | undefined
) {
  const defaultInstitutionId = getDefaultInstitutionId?.();
  if (typeof defaultInstitutionId === "number") return defaultInstitutionId;

  const firstInstitutionValue = institutionItems.value.find((institution) => (
    typeof institution.value === "number"
  ))?.value;
  const firstInstitutionId = typeof firstInstitutionValue === "number"
    ? firstInstitutionValue
    : undefined;

  return firstInstitutionId;
}

function normalizeInstitutionName(name: string) {
  return name.trim().toLowerCase();
}

export function useAccountUpsertForm(params: UseAccountUpsertFormParams) {
  const defaults: RequiredOrUndefined<AccountFormInputValues> = {
    account_type: "current",
    currency_code: "GBP",
    institution: undefined,
    name: undefined,
    normal_balance_sign: 1,
    opened_date: undefined
  };

  const state = shallowReactive<Partial<AccountFormInputValues>>({
    ...defaults
  });

  const institutionItems = computed(() => {
    const institutions = params.institutions.value ?? [];
    const items = institutions.map((institution): { label: string, value: number | string } => ({
      label: institution.name,
      value: institution.id
    }));

    if (state.institution?.kind !== "new") return items;

    const createdInstitutionName = state.institution.input.name.trim();
    if (!createdInstitutionName) return items;

    const createdInstitutionExists = items.some((institution) => (
      normalizeInstitutionName(institution.label) === normalizeInstitutionName(createdInstitutionName)
    ));
    if (createdInstitutionExists) return items;

    return [
      ...items,
      { label: createdInstitutionName, value: createdInstitutionName }
    ];
  });

  function setExistingInstitution(id: number | undefined) {
    state.institution = id == null ? undefined : { kind: "existing", id };
  }

  function findExistingInstitution(name: string) {
    const normalized = normalizeInstitutionName(name);
    if (!normalized) return undefined;

    return (params.institutions.value ?? []).find((institution) => (
      normalizeInstitutionName(institution.name) === normalized
    ));
  }

  function findExistingInstitutionId(name: string) {
    return findExistingInstitution(name)?.id;
  }

  function onInstitutionCreate(name: string) {
    const trimmedName = name.trim();
    if (!trimmedName) return;

    const existingInstitutionId = findExistingInstitutionId(trimmedName);
    if (existingInstitutionId != null) {
      setExistingInstitution(existingInstitutionId);
      return;
    }

    state.institution = {
      kind: "new",
      input: { name: trimmedName }
    };
  }

  const institutionSearchTerm = ref("");

  const institutionCreateItem = computed<false | "always">(() => {
    const trimmedSearchTerm = institutionSearchTerm.value.trim();
    if (!trimmedSearchTerm) return false;
    return findExistingInstitution(trimmedSearchTerm) ? false : "always";
  });

  function onInstitutionSearchTermUpdate(searchTerm: string) {
    institutionSearchTerm.value = searchTerm;
  }

  const institutionMenuValue = computed<number | string | undefined>({
    get: () => {
      if (state.institution?.kind === "existing") return state.institution.id;
      if (state.institution?.kind === "new") return state.institution.input.name;
      return undefined;
    },
    set: (value) => {
      if (typeof value === "number") {
        setExistingInstitution(value);
        return;
      }

      if (typeof value === "string") {
        onInstitutionCreate(value);
        return;
      }

      state.institution = undefined;
    }
  });

  const accountTypeItems = computed(() => {
    const values: AccountTypeName[] = [
      "current",
      "savings",
      "credit_card",
      "isa",
      "investment",
      "pension",
      "cash",
      "loan"
    ];
    return values.map((value) => ({
      label: ACCOUNT_TYPE_META[value].label,
      value
    }));
  });

  const normalBalanceSignItems = [
    { label: "Positive (1)", value: 1 },
    { label: "Negative (-1)", value: -1 }
  ];

  function reset() {
    Object.assign(state, defaults);
    setExistingInstitution(defaultInstitutionFallback(institutionItems, params.getDefaultInstitutionId));
    institutionSearchTerm.value = "";
  }

  function hydrateFromAccount(account: AccountDto) {
    state.institution = { kind: "existing", id: account.institution.id };
    state.name = account.name;
    state.account_type = account.account_type.name;
    state.currency_code = account.currency_code;
    state.normal_balance_sign = account.normal_balance_sign === -1 ? -1 : 1;
    state.opened_date = account.opened_date == null ? undefined : parseDate(account.opened_date);
    institutionSearchTerm.value = "";
  }

  watch(() => state.account_type, (kind) => {
    if (!kind) return;
    state.normal_balance_sign = kind === "credit_card" || kind === "loan" ? -1 : 1;
  });

  watch(institutionItems, (items) => {
    if (state.institution?.kind === "new") {
      const createdInstitutionId = findExistingInstitutionId(state.institution.input.name);
      if (createdInstitutionId != null) {
        setExistingInstitution(createdInstitutionId);
      }
      return;
    }

    if (state.institution?.kind === "existing" && state.institution.id != null) return;
    const firstInstitutionId = items.find((institution) => (
      typeof institution.value === "number"
    ))?.value;
    setExistingInstitution(typeof firstInstitutionId === "number" ? firstInstitutionId : undefined);
  }, { immediate: true });

  return {
    state,
    institutionItems,
    institutionMenuValue,
    institutionCreateItem,
    onInstitutionSearchTermUpdate,
    onInstitutionCreate,
    accountTypeItems,
    normalBalanceSignItems,
    reset,
    hydrateFromAccount
  };
}
