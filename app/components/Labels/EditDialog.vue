<template>
  <UModal
    v-model:open="open"
    :title="label ? 'Edit label' : 'Create label'"
    :dismissible="!isPending && !isDirty"
    :close="!isPending"
  >
    <template #body>
      <UForm
        ref="form"
        :schema="labelUpsertInputGeneratedSchema"
        :state="state"
        :validate-on="['blur']"
        class="space-y-4"
        @submit="onSubmit"
      >
        <UAlert v-if="submitError" color="error" variant="subtle" :title="submitError" />
        <UFormField label="Name" name="name">
          <UInput v-model.trim="state.name" placeholder="e.g. ISA" class="w-full" autofocus :disabled="isPending" />
        </UFormField>
        <UFormField label="Description" name="description" hint="Optional">
          <UTextarea v-model="state.description" class="w-full" :rows="5" :disabled="isPending" />
        </UFormField>
        <div class="flex justify-end gap-2">
          <UButton color="neutral" variant="ghost" :disabled="isPending" @click="open = false">
            Cancel
          </UButton>
          <UButton type="submit" :loading="isPending">
            {{ label ? 'Save changes' : 'Create label' }}
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { FormSubmitEvent } from "@nuxt/ui";
import type { ComponentExposed } from "vue-component-type-helpers";
import type { UForm } from "#components";
import type { LabelDto, LabelUpsertInput } from "~/generated/bindings";
import { labelUpsertInputGeneratedSchema } from "~/generated/zod";

const props = defineProps<{ label: LabelDto | null }>();
const open = defineModel<boolean>("open", { required: true });
const state = reactive({ name: "", description: "" });
const initialName = ref("");
const initialDescription = ref("");
const isDirty = computed(() => state.name !== initialName.value || state.description !== initialDescription.value);
const submitError = ref<string | null>(null);
const form = useTemplateRef<ComponentExposed<typeof UForm<typeof labelUpsertInputGeneratedSchema>>>("form");
const setBackendValidationErrors = useBackendValidationErrors(form);
const { createLabel, updateLabel } = useLabelMutations();
const isPending = computed(() => createLabel.isPending || updateLabel.isPending);

useNavigationLayer({
  id: "label-edit-dialog",
  open,
  dirty: isDirty,
  pending: isPending,
  close: () => {
    open.value = false;
  },
  discardTitle: "Discard label changes?"
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  initialName.value = props.label?.name ?? "";
  state.name = initialName.value;
  initialDescription.value = props.label?.description ?? "";
  state.description = initialDescription.value;
  submitError.value = null;
  form.value?.clear();
});

async function onSubmit(event: FormSubmitEvent<LabelUpsertInput>) {
  submitError.value = null;
  try {
    const description = event.data.description?.trim() ?? "";
    const input = { name: event.data.name.trim(), description: description.length > 0 ? description : null };
    if (props.label) {
      await updateLabel.mutateAsync({ labelId: props.label.id, input });
    } else {
      await createLabel.mutateAsync(input);
    }
    open.value = false;
  } catch (error) {
    if (!setBackendValidationErrors(error)) {
      submitError.value = error instanceof Error ? error.message : "Could not save label";
    }
  }
}
</script>
