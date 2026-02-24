<template>
  <UModal
    v-model:open="open"
    title="Create institution"
    description="Add a new institution to your account list."
    :dismissible="!form?.loading"
    :close="!form?.loading"
  >
    <template #body>
      <UForm
        ref="form"
        :schema="institutionFormSchema"
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

        <UFormField label="Institution name" name="name">
          <UInput
            v-model="state.name"
            placeholder="e.g. Barclays"
            class="w-full"
            autofocus
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
            Create institution
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { FormSubmitEvent } from "@nuxt/ui";

const open = defineModel<boolean>("open", { required: true });
const submitError = ref<string | null>(null);
const form = useTemplateRef("form");
const setBackendValidationErrors = useBackendValidationErrors(form);
const { state, reset } = useInstitutionUpsertForm();

const { createInstitution } = useInstitutionMutations();

watch(open, (isOpen) => {
  if (isOpen) {
    submitError.value = null;
    reset();
    form.value?.clear();
  }
});

async function onSubmit(event: FormSubmitEvent<InstitutionFormValues>) {
  submitError.value = null;

  try {
    const { id } = await createInstitution.mutateAsync({
      name: event.data.name
    });

    open.value = false;
    await navigateTo(`/institutions/${id}`);
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to create institution";
    }
  }
}
</script>
