<template>
  <UModal
    v-model:open="open"
    title="Create account"
  >
    <template #body>
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
              placeholder="e.g. Nationwide"
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
            placeholder="e.g. Everyday Current"
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
              placeholder="GBP"
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

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :disabled="isSubmitting"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            type="submit"
            color="primary"
            :loading="isSubmitting"
            :disabled="isSubmitting"
          >
            Create account
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { FormError, FormSubmitEvent } from "@nuxt/ui";
import type { AccountTypeName, AccountUpsertInput, InstitutionSummaryDto } from "~/bindings";
import type { AccountFormValues } from "~/utils/forms/schemas";

import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

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

const props = withDefaults(defineProps<{
  defaultInstitutionId?: number | null
}>(), {
  defaultInstitutionId: null
});

const open = defineModel<boolean>("open", { required: true });
const form = useTemplateRef<{ clear: () => void, setErrors: (errors: FormError[]) => void }>("form");

const api = useApi();
const { createAccount } = useAccountMutations();

const submitError = ref<string | null>(null);

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

const state = reactive<AccountFormState>({
  institution: props.defaultInstitutionId != null
    ? { kind: "existing", id: props.defaultInstitutionId }
    : undefined,
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

    const fallback = props.defaultInstitutionId ?? institutionItems.value[0]?.value;
    setExistingInstitution(fallback);
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

const isSubmitting = computed(() => createAccount.isPending.value);

watch(open, (isOpen) => {
  if (!isOpen) return;

  submitError.value = null;
  setExistingInstitution(props.defaultInstitutionId ?? undefined);
  state.name = "";
  state.account_type = "current";
  state.currency_code = "GBP";
  state.normal_balance_sign = 1;
  state.opened_date = "";
  form.value?.clear();
});

watch(institutionItems, (items) => {
  if (state.institution?.kind === "new") return;
  if (state.institution?.kind === "existing" && state.institution.id != null) return;
  setExistingInstitution(items[0]?.value);
}, { immediate: true });

watch(() => state.account_type, (kind) => {
  if (!kind) return;
  state.normal_balance_sign = kind === "credit_card" || kind === "loan" ? -1 : 1;
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
  submitError.value = null;
  const payload: AccountUpsertInput = {
    ...event.data,
    opened_date: event.data.opened_date ?? null
  };

  try {
    const account = await createAccount.mutateAsync(payload);
    open.value = false;
    await navigateTo(`/accounts/${account.id}`);
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to create account";
    }
  }
}
</script>
