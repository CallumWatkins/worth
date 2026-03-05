<template>
  <UContainer>
    <div v-if="accountQuery.isSuccess" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="accountQuery.isSuccess"
      title="Account settings"
      :description="`${accountQuery.data.institution.name} • ${accountQuery.data.name}`"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />

    <UPageBody class="space-y-6">
      <template v-if="accountQuery.isSuccess">
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
              v-if="institutionsQuery.isError.value"
              color="error"
              variant="subtle"
              :title="institutionsQuery.error.value?.message ?? 'Failed to load institutions'"
            />

            <UFormField
              label="Institution"
              name="institution"
              :error-pattern="/^institution(\..*)?$/"
            >
              <USelectMenu
                v-model="institutionMenuValue"
                :items="institutionItems"
                value-key="value"
                :create-item="institutionCreateItem"
                placeholder="Select or create institution"
                class="w-full"
                :loading="institutionsQuery.isPending.value"
                :disabled="institutionsQuery.isPending.value"
                :ui="{
                  base: typeof institutionMenuValue === 'string' ? 'ps-13' : 'ps-2.5',
                  leading: typeof institutionMenuValue === 'string' ? undefined : 'hidden'
                }"
                @update:search-term="onInstitutionSearchTermUpdate"
                @create="onInstitutionCreate"
              >
                <template #leading>
                  <UBadge
                    color="neutral"
                    variant="soft"
                    size="sm"
                  >
                    New
                  </UBadge>
                </template>
                <template #item-leading="item">
                  <UBadge
                    v-if="typeof item.item.value === 'string'"
                    color="neutral"
                    variant="soft"
                    size="sm"
                  >
                    New
                  </UBadge>
                </template>
                <template #create-item-label="{ item }">
                  Create new institution "{{ item }}"
                </template>
              </USelectMenu>
            </UFormField>

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
              />
            </UFormField>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <UFormField label="Currency" name="currency_code">
                <USelectMenu
                  v-model="state.currency_code"
                  :items="supportedCurrencyCodes"
                  placeholder="Select currency"
                  class="w-full"
                />
              </UFormField>

              <UFormField label="Normal balance sign" name="normal_balance_sign">
                <USelect
                  v-model="state.normal_balance_sign"
                  :items="normalBalanceSignItems"
                  class="w-full"
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

            <div class="flex items-center justify-end gap-3">
              <Transition name="save-status-fade">
                <span v-if="didSave && !form?.dirty" class="text-sm text-success">
                  Changes saved
                </span>
              </Transition>
              <UButton
                type="submit"
                color="primary"
                :disabled="form?.loading"
              >
                Save changes
              </UButton>
            </div>
          </UForm>
        </UPageCard>

        <UPageCard
          title="Actions"
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
              @click="deleteOpen = true"
            >
              Delete account
            </UButton>
          </div>
        </UPageCard>

        <AccountsDeleteDialog
          v-model:open="deleteOpen"
          :account-id="accountId"
        />
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem, FormSubmitEvent } from "@nuxt/ui";
import type { AccountUpsertInput } from "~/generated/bindings";
import type { AccountBreadcrumbContext } from "~/middleware/accountBreadcrumbContext.global";
import { useQuery } from "@tanstack/vue-query";
import { supportedCurrencyCodes } from "~/utils/currencies";

const route = useRoute("accounts-id-settings");
const api = useApi();
const accountBreadcrumbContext = useState<AccountBreadcrumbContext | null>("accountBreadcrumbContext", () => null);
const { updateAccount } = useAccountMutations();
const form = useTemplateRef("form");
const setBackendValidationErrors = useBackendValidationErrors(form);

const accountId = useRouteParamInt(route, "id");

const accountQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.get(accountId.value!)),
  enabled: computed(() => accountId.value !== null),
  queryFn: () => api.accountsGet(accountId.value!)
}));

useResourcePageError({
  resourceName: "Account",
  resourceId: accountId,
  resourceIsError: computed(() => accountQuery.isError),
  resourceError: computed(() => accountQuery.error),
  fallbackErrorMessage: "Failed to load account"
});

const institutionsQuery = useQuery({
  queryKey: queryKeys.institutions.list(),
  enabled: computed(() => !!accountQuery.data),
  queryFn: api.institutionsList
});

const submitError = ref<string | null>(null);
const didSave = ref(false);
const hasHydrated = ref(false);
const deleteOpen = ref(false);
const {
  state,
  institutionItems,
  institutionMenuValue,
  institutionCreateItem,
  onInstitutionSearchTermUpdate,
  onInstitutionCreate,
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
  didSave.value = false;
});

usePreventRouteNavigation({
  isSubmitting: computed(() => form.value?.loading ?? false),
  isDirty: computed(() => form.value?.dirty ?? false),
  title: "Discard account changes?",
  description: "You have unsaved account changes that will be lost if you leave this page."
});

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const account = accountQuery.data;
  if (!account) return [];

  const context = accountBreadcrumbContext.value;
  if (context && context.accountId === account.id && context.institutionId === account.institution.id) {
    return [
      { label: "Institutions", to: { name: "institutions" }, icon: "i-lucide-building-2" },
      { label: account.institution.name, to: { name: "institutions-id", params: { id: account.institution.id } } },
      { label: account.name, to: { name: "accounts-id", params: { id: account.id } } },
      { label: "Settings" }
    ];
  }

  return [
    { label: "Accounts", to: { name: "accounts" }, icon: "i-lucide-wallet" },
    { label: account.name, to: { name: "accounts-id", params: { id: account.id } } },
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
    throw error;
  }
}
</script>
