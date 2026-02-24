<template>
  <UContainer>
    <div v-if="accountQuery.data" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="accountQuery.data"
      title="Account settings"
      :description="accountQuery.data.name"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />

    <UPageBody class="space-y-6">
      <UAlert
        v-if="invalidId"
        color="error"
        variant="subtle"
        title="Invalid account id"
      />

      <UAlert
        v-else-if="accountQuery.isError"
        color="error"
        variant="subtle"
        :title="accountQuery.error!.message ?? 'Failed to load account'"
      />

      <template v-else-if="accountQuery.data">
        <UPageCard title="General" description="Update account details and institution">
          <UForm
            ref="form"
            :schema="accountFormSchema"
            :state="state"
            :validate-on="['blur']"
            class="space-y-4"
            @submit="onSubmit"
          >
            <UAlert
              v-if="submitError"
              color="error"
              variant="subtle"
              :title="submitError"
            />

            <UAlert
              v-if="didSave"
              color="success"
              variant="subtle"
              title="Account updated"
            />

            <UAlert
              v-if="institutionsQuery.isError.value"
              color="error"
              variant="subtle"
              :title="institutionsQuery.error.value?.message ?? 'Failed to load institutions'"
            />

            <div class="space-y-2">
              <label class="flex items-center gap-2 text-sm text-muted">
                <UCheckbox v-model="createNewInstitution" color="neutral" />
                <span>Create new institution</span>
              </label>

              <UFormField
                v-if="createNewInstitution"
                label="New institution name"
                name="institution.input.name"
                :error-pattern="/^institution(\.input\.name)?$/"
              >
                <UInput
                  v-model="newInstitutionName"
                  class="w-full"
                  :disabled="isSubmitting"
                />
              </UFormField>

              <UFormField
                v-else
                label="Institution"
                name="institution.id"
                :error-pattern="/^institution(\.id)?$/"
              >
                <USelect
                  v-model="selectedInstitutionId"
                  :items="institutionItems"
                  class="w-full"
                  color="neutral"
                  variant="subtle"
                  :content="{ bodyLock: false }"
                  :disabled="isSubmitting"
                />
              </UFormField>
            </div>

            <UFormField label="Account name" name="name">
              <UInput
                v-model="state.name"
                class="w-full"
                :disabled="isSubmitting"
              />
            </UFormField>

            <UFormField label="Account type" name="account_type">
              <USelect
                v-model="state.account_type"
                :items="accountTypeItems"
                class="w-full"
                color="neutral"
                variant="subtle"
                :content="{ bodyLock: false }"
                :disabled="isSubmitting"
              />
            </UFormField>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <UFormField label="Currency" name="currency_code">
                <UInput
                  v-model="state.currency_code"
                  class="w-full"
                  :disabled="isSubmitting"
                  maxlength="3"
                  @blur="state.currency_code = state.currency_code?.trim().toUpperCase() ?? ''"
                />
              </UFormField>

              <UFormField label="Normal balance sign" name="normal_balance_sign">
                <USelect
                  v-model="state.normal_balance_sign"
                  :items="normalBalanceSignItems"
                  class="w-full"
                  color="neutral"
                  variant="subtle"
                  :content="{ bodyLock: false }"
                  :disabled="isSubmitting"
                />
              </UFormField>
            </div>

            <UFormField label="Opened date (optional)" name="opened_date">
              <UInput
                v-model="openedDateInput"
                type="date"
                class="w-full"
                :disabled="isSubmitting"
              />
            </UFormField>

            <div class="flex justify-end">
              <UButton
                type="submit"
                color="primary"
                :loading="isSubmitting"
                :disabled="isSubmitting || !isDirty"
              >
                Save changes
              </UButton>
            </div>
          </UForm>
        </UPageCard>

        <UPageCard
          title="Actions"
          description="Additional actions are planned for this page."
        >
          <div class="flex flex-wrap gap-2">
            <UButton
              color="neutral"
              variant="subtle"
              icon="i-lucide-download"
              disabled
            >
              Export (coming soon)
            </UButton>
            <UButton
              color="error"
              variant="subtle"
              icon="i-lucide-trash-2"
              disabled
            >
              Delete (coming soon)
            </UButton>
          </div>
        </UPageCard>
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem, FormError, FormSubmitEvent } from "@nuxt/ui";
import type {
  AccountTypeName,
  AccountUpsertInput,
  InstitutionSummaryDto
} from "~/bindings";
import type { AccountFormValues } from "~/utils/forms/schemas";

import { useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs } from "vue";

import { accountTypeLabel } from "~/utils/account-type-meta";
import { accountFormSchema } from "~/utils/forms/schemas";

interface AccountFormState {
  institution?: AccountFormValues["institution"]
  name?: string
  account_type?: AccountTypeName
  currency_code?: string
  normal_balance_sign?: 1 | -1
  opened_date?: string | null
}

const route = useRoute();
const api = useApi();
const { updateAccount } = useAccountMutations();
const form = useTemplateRef<{ clear: () => void, setErrors: (errors: FormError[]) => void }>("form");

const rawId = computed(() => {
  const p = (route.params as any)?.id;
  if (Array.isArray(p)) return p[0];
  return p;
});

const accountId = computed<number | null>(() => {
  const s = String(rawId.value ?? "");
  const n = Number.parseInt(s, 10);
  if (!Number.isFinite(n)) return null;
  return n;
});

const invalidId = computed(() => rawId.value != null && accountId.value == null);

const accountQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.get(accountId.value!)),
  enabled: computed(() => accountId.value !== null),
  queryFn: () => api.accountsGet(accountId.value!)
}));

const institutionsQuery = useQuery({
  queryKey: queryKeys.institutions.list(),
  queryFn: api.institutionsList
});

const institutionItems = computed(() => {
  const institutions = institutionsQuery.data.value ?? [];
  return institutions.map((institution: InstitutionSummaryDto) => ({
    label: institution.name,
    value: institution.id
  }));
});

const submitError = ref<string | null>(null);
const didSave = ref(false);
const hasHydrated = ref(false);
const state = reactive<AccountFormState>({
  institution: undefined,
  name: "",
  account_type: "current",
  currency_code: "GBP",
  normal_balance_sign: 1,
  opened_date: ""
});

function setExistingInstitution(id: number | undefined) {
  state.institution = id == null ? undefined : { kind: "existing", id };
}

const createNewInstitution = computed({
  get: () => state.institution?.kind === "new",
  set: (value: boolean) => {
    if (value) {
      state.institution = { kind: "new", input: { name: "" } } as AccountFormValues["institution"];
      return;
    }
    setExistingInstitution(institutionItems.value[0]?.value);
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
    state.institution = { kind: "new", input: { name: value } } as AccountFormValues["institution"];
  }
});

const openedDateInput = computed<string>({
  get: () => state.opened_date ?? "",
  set: (value: string) => {
    state.opened_date = value;
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

watch(() => accountQuery.data, (account) => {
  if (!account || hasHydrated.value) return;
  state.institution = { kind: "existing", id: account.institution.id };
  state.name = account.name;
  state.account_type = account.account_type.name;
  state.currency_code = account.currency_code;
  state.normal_balance_sign = account.normal_balance_sign === -1 ? -1 : 1;
  state.opened_date = account.opened_date ?? "";
  hasHydrated.value = true;
  form.value?.clear();
}, { immediate: true });

watch(accountId, () => {
  hasHydrated.value = false;
});

watch(() => state.account_type, (kind) => {
  if (!kind) return;
  state.normal_balance_sign = kind === "credit_card" || kind === "loan" ? -1 : 1;
});

watch(institutionItems, (items) => {
  if (state.institution?.kind === "new") return;
  if (state.institution?.kind === "existing" && state.institution.id != null) return;
  setExistingInstitution(items[0]?.value);
}, { immediate: true });

const isSubmitting = computed(() => updateAccount.isPending.value);

const isDirty = computed(() => {
  const account = accountQuery.data;
  if (!account) return false;

  if ((state.name ?? "") !== account.name) return true;
  if ((state.account_type ?? "current") !== account.account_type.name) return true;
  if ((state.currency_code ?? "GBP") !== account.currency_code) return true;
  if ((state.normal_balance_sign ?? 1) !== account.normal_balance_sign) return true;
  if (((state.opened_date ?? "") || null) !== account.opened_date) return true;

  if (state.institution?.kind === "new") {
    return !!state.institution.input.name?.trim();
  }

  return state.institution?.id !== account.institution.id;
});

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const account = accountQuery.data;
  if (!account) return [];
  return [
    { label: "Accounts", to: "/accounts", icon: "i-lucide-wallet" },
    { label: account.name, to: `/accounts/${account.id}` },
    { label: "Settings" }
  ];
});

function setBackendValidationErrors(error: unknown) {
  const issues = extractValidationIssues(error);
  if (!issues.length) return false;
  form.value?.setErrors(issues.map((issue) => ({
    name: issue.field,
    message: issue.message
  })));
  return true;
}

async function onSubmit(event: FormSubmitEvent<AccountFormValues>) {
  if (!accountId.value) return;

  submitError.value = null;
  didSave.value = false;
  const payload: AccountUpsertInput = {
    ...event.data,
    opened_date: event.data.opened_date ?? null
  };

  try {
    await updateAccount.mutateAsync({
      accountId: accountId.value,
      input: payload
    });
    didSave.value = true;
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to update account";
    }
  }
}
</script>
