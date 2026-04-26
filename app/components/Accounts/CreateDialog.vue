<template>
  <UModal
    v-model:open="open"
    title="Create account"
    :dismissible="!form?.loading && !form?.dirty"
    :close="!form?.loading"
  >
    <template #body>
      <UForm
        ref="form"
        :schema="accountFormSchema"
        :state="state"
        :validate-on="['blur']"
        :validate-on-input-delay="0"
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
          v-if="institutionsQuery.isError"
          color="error"
          variant="subtle"
          :title="institutionsQuery.error.message"
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
            :loading="institutionsQuery.isPending"
            :disabled="institutionsQuery.isPending"
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
            placeholder="e.g. Everyday Current"
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

        <div class="grid grid-cols-2 gap-3">
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

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :disabled="form?.loading"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            type="submit"
            loading-auto
          >
            Create account
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { UForm } from "#components";
import type { FormSubmitEvent } from "@nuxt/ui";
import type { ComponentExposed } from "vue-component-type-helpers";
import type { AccountUpsertInput } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";
import { supportedCurrencyCodes } from "~/utils/currencies";

const props = withDefaults(defineProps<{
  defaultInstitutionId?: number | null
}>(), {
  defaultInstitutionId: null
});

const open = defineModel<boolean>("open", { required: true });
const form = useTemplateRef<ComponentExposed<typeof UForm<typeof accountFormSchema>>>("form");

const api = useApi();
const { createAccount } = useAccountMutations();
const setBackendValidationErrors = useBackendValidationErrors(form);

const submitError = ref<string | null>(null);

const institutionsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.institutions.list(),
  queryFn: api.institutionsList
}));

const {
  state,
  institutionItems,
  institutionMenuValue,
  institutionCreateItem,
  onInstitutionSearchTermUpdate,
  onInstitutionCreate,
  accountTypeItems,
  normalBalanceSignItems,
  reset
} = useAccountUpsertForm({
  institutions: computed(() => institutionsQuery.data),
  getDefaultInstitutionId: () => props.defaultInstitutionId
});

watch(open, (isOpen) => {
  if (!isOpen) return;

  submitError.value = null;
  reset();
  form.value?.clear();
});

async function onSubmit(event: FormSubmitEvent<AccountFormValues>) {
  submitError.value = null;
  const payload: AccountUpsertInput = event.data;

  try {
    const { id } = await createAccount.mutateAsync(payload);
    open.value = false;
    await navigateTo({ name: "accounts-id", params: { id } });
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to create account";
    }
  }
}
</script>
