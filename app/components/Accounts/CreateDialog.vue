<template>
  <UModal
    v-model:open="open"
    title="Create account"
    :dismissible="!form?.loading"
    :close="!form?.loading"
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
              placeholder="e.g. Nationwide"
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
            placeholder="e.g. Everyday Current"
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
              placeholder="GBP"
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
import type { FormSubmitEvent } from "@nuxt/ui";
import type { AccountUpsertInput } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

const props = withDefaults(defineProps<{
  defaultInstitutionId?: number | null
}>(), {
  defaultInstitutionId: null
});

const open = defineModel<boolean>("open", { required: true });
const form = useTemplateRef("form");

const api = useApi();
const { createAccount } = useAccountMutations();
const setBackendValidationErrors = useBackendValidationErrors(form);

const submitError = ref<string | null>(null);

const institutionsQuery = useQuery({
  queryKey: queryKeys.institutions.list(),
  queryFn: api.institutionsList
});

const {
  state,
  institutionItems,
  createNewInstitution,
  selectedInstitutionId,
  newInstitutionName,
  accountTypeItems,
  normalBalanceSignItems,
  reset
} = useAccountUpsertForm({
  institutions: computed(() => institutionsQuery.data.value),
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
