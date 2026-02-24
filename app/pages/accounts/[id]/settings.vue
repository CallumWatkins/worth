<template>
  <UContainer>
    <div v-if="accountQuery.data" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="accountQuery.data"
      title="Account settings"
      :description="`${accountQuery.data.institution.name} • ${accountQuery.data.name}`"
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
                <UCheckbox v-model="createNewInstitution" />
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
                  :content="{ bodyLock: false }"
                />
              </UFormField>
            </div>

            <UFormField label="Account name" name="name">
              <UInput
                v-model="state.name"
                class="w-full"
              />
            </UFormField>

            <UFormField label="Account type" name="account_type">
              <USelect
                v-model="state.account_type"
                :items="accountTypeItems"
                class="w-full"
                :content="{ bodyLock: false }"
              />
            </UFormField>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <UFormField label="Currency" name="currency_code">
                <UInput
                  v-model="state.currency_code"
                  class="w-full"
                  maxlength="3"
                  @blur="state.currency_code = state.currency_code?.trim().toUpperCase() ?? ''"
                />
              </UFormField>

              <UFormField label="Normal balance sign" name="normal_balance_sign">
                <USelect
                  v-model="state.normal_balance_sign"
                  :items="normalBalanceSignItems"
                  class="w-full"
                  :content="{ bodyLock: false }"
                />
              </UFormField>
            </div>

            <UFormField label="Opened date (optional)" name="opened_date">
              <UInputDate
                v-model="state.opened_date"
                :range="false"
                class="w-full"
              />
            </UFormField>

            <div class="flex justify-end">
              <UButton
                type="submit"
                color="primary"
                :loading="isSubmitting"
                :disabled="isSubmitting"
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
import type { BreadcrumbItem, FormSubmitEvent } from "@nuxt/ui";
import type { AccountUpsertInput } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

const route = useRoute();
const api = useApi();
const { updateAccount } = useAccountMutations();
const form = useTemplateRef("form");
const setBackendValidationErrors = useBackendValidationErrors(form);

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

const submitError = ref<string | null>(null);
const didSave = ref(false);
const hasHydrated = ref(false);
const {
  state,
  institutionItems,
  createNewInstitution,
  selectedInstitutionId,
  newInstitutionName,
  accountTypeItems,
  normalBalanceSignItems,
  hydrateFromAccount
} = useAccountUpsertForm({
  institutions: computed(() => institutionsQuery.data.value)
});

watch(() => accountQuery.data, (account) => {
  if (!account || hasHydrated.value) return;
  hydrateFromAccount(account);
  hasHydrated.value = true;
  form.value?.clear();
}, { immediate: true });

watch(accountId, () => {
  hasHydrated.value = false;
});

const isSubmitting = computed(() => updateAccount.isPending.value);

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const account = accountQuery.data;
  if (!account) return [];
  return [
    { label: "Accounts", to: "/accounts", icon: "i-lucide-wallet" },
    { label: account.name, to: `/accounts/${account.id}` },
    { label: "Settings" }
  ];
});

async function onSubmit(event: FormSubmitEvent<AccountFormValues>) {
  if (!accountId.value) return;

  submitError.value = null;
  didSave.value = false;
  const payload: AccountUpsertInput = event.data;

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
