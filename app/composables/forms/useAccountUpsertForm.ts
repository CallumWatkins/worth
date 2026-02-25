import type { AccountDto, AccountTypeName, InstitutionSummaryDto } from "~/generated/bindings";
import { parseDate } from "@internationalized/date";

interface UseAccountUpsertFormParams {
  institutions: Ref<InstitutionSummaryDto[] | undefined>
  getDefaultInstitutionId?: () => number | null | undefined
}

function defaultInstitutionFallback(
  institutionItems: { value: { label: string, value: number }[] },
  getDefaultInstitutionId?: () => number | null | undefined
) {
  return getDefaultInstitutionId?.() ?? institutionItems.value[0]?.value;
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
    return institutions.map((institution) => ({
      label: institution.name,
      value: institution.id
    }));
  });

  function setExistingInstitution(id: number | undefined) {
    state.institution = id == null ? undefined : { kind: "existing", id };
  }

  const createNewInstitution = computed({
    get: () => state.institution?.kind === "new",
    set: (value: boolean) => {
      if (value) {
        state.institution = { kind: "new", input: { name: "" } };
        return;
      }
      setExistingInstitution(defaultInstitutionFallback(institutionItems, params.getDefaultInstitutionId));
    }
  });

  const selectedInstitutionId = computed<number | undefined>({
    get: () => state.institution?.kind === "existing" ? state.institution.id : undefined,
    set: (value: number | undefined) => {
      setExistingInstitution(value);
    }
  });

  const newInstitutionName = computed<string>({
    get: () => state.institution?.kind === "new" ? (state.institution.input.name ?? "") : "",
    set: (value: string) => {
      state.institution = { kind: "new", input: { name: value } };
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
      label: accountTypeLabel(value),
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
  }

  function hydrateFromAccount(account: AccountDto) {
    state.institution = { kind: "existing", id: account.institution.id };
    state.name = account.name;
    state.account_type = account.account_type.name;
    state.currency_code = account.currency_code;
    state.normal_balance_sign = account.normal_balance_sign === -1 ? -1 : 1;
    state.opened_date = account.opened_date == null ? undefined : parseDate(account.opened_date);
  }

  watch(() => state.account_type, (kind) => {
    if (!kind) return;
    state.normal_balance_sign = kind === "credit_card" || kind === "loan" ? -1 : 1;
  });

  watch(institutionItems, (items) => {
    if (state.institution?.kind === "new") return;
    if (state.institution?.kind === "existing" && state.institution.id != null) return;
    setExistingInstitution(items[0]?.value);
  }, { immediate: true });

  return {
    state,
    institutionItems,
    createNewInstitution,
    selectedInstitutionId,
    newInstitutionName,
    accountTypeItems,
    normalBalanceSignItems,
    reset,
    hydrateFromAccount
  };
}
