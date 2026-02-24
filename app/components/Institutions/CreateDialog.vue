<template>
  <UModal
    v-model:open="open"
    title="Create institution"
    description="Add a new institution to your account list."
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
            :disabled="isSubmitting"
            autofocus
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
            Create institution
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { FormError, FormSubmitEvent } from "@nuxt/ui";
import type { InstitutionFormValues } from "~/utils/forms/schemas";

import { computed } from "vue";

import { institutionFormSchema } from "~/utils/forms/schemas";

const open = defineModel<boolean>("open", { required: true });
const submitError = ref<string | null>(null);
const form = useTemplateRef<{ clear: () => void, setErrors: (errors: FormError[]) => void }>("form");
const state = reactive<Partial<InstitutionFormValues>>({
  name: ""
});

const { createInstitution } = useInstitutionMutations();

const isSubmitting = computed(() => createInstitution.isPending.value);

watch(open, (isOpen) => {
  if (isOpen) {
    submitError.value = null;
    state.name = "";
    form.value?.clear();
  }
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

async function onSubmit(event: FormSubmitEvent<InstitutionFormValues>) {
  submitError.value = null;

  try {
    const institution = await createInstitution.mutateAsync({
      name: event.data.name
    });

    open.value = false;
    await navigateTo(`/institutions/${institution.id}`);
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Failed to create institution";
    }
  }
}
</script>
