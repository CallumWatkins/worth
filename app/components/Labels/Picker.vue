<template>
  <div class="space-y-2">
    <UInputMenu
      :id="id"
      v-model="selection"
      v-model:search-term="searchTerm"
      v-bind="ariaAttrs"
      :items="items"
      value-key="value"
      open-on-focus
      multiple
      placeholder="Select or create labels"
      class="w-full"
      :color="color"
      :highlight="highlight"
      :loading="labelsQuery.isPending"
      :disabled="disabled || labelsQuery.isPending || labelsQuery.isError"
      :create-item="canCreate ? 'always' : false"
      :ui="{ base: 'flex-wrap', tagsItem: 'max-w-full', tagsItemText: 'truncate', content: 'max-h-64' }"
      @create="onCreate"
      @blur="emitFormBlur"
      @focus="emitFormFocus"
    >
      <template #item-trailing="{ item }">
        <UBadge v-if="typeof item.value === 'string'" color="neutral" variant="soft" size="sm">
          New
        </UBadge>
      </template>
      <template #create-item-label>
        Create “{{ searchTerm.trim() }}”
      </template>
      <template #empty>
        {{ searchTerm ? 'No matching labels' : 'Type to create your first label' }}
      </template>
    </UInputMenu>
    <p v-if="searchTerm.trim() && !newLabelValidation.success" class="text-sm text-error" role="status">
      {{ newLabelValidation.error.issues[0]?.message }}
    </p>
    <UAlert v-if="labelsQuery.isError" color="error" variant="subtle" title="Could not load labels">
      <template #actions>
        <UButton color="neutral" variant="subtle" size="xs" :loading="labelsQuery.isFetching" @click="labelsQuery.refetch()">
          Retry
        </UButton>
      </template>
    </UAlert>
  </div>
</template>

<script setup lang="ts">
import type { LabelRef } from "~/generated/bindings";
import { useFormField } from "@nuxt/ui/composables/useFormField";
import { useQuery } from "@tanstack/vue-query";
import { labelUpsertInputGeneratedSchema } from "~/generated/zod";

const props = defineProps<{ disabled?: boolean }>();
const model = defineModel<LabelRef[]>({ default: () => [] });
const { id, ariaAttrs, disabled, color, highlight, emitFormInput, emitFormChange, emitFormBlur, emitFormFocus } = useFormField(props);
const api = useApi();
const labelsQuery = proxyRefs(useQuery({ queryKey: queryKeys.labels.list(), queryFn: api.labelsList }));
const searchTerm = ref("");

const items = computed(() => [
  ...(labelsQuery.data ?? []).map((label) => ({ label: label.name, value: label.id })),
  ...model.value.flatMap((label) => label.kind === "new"
    ? [{ label: label.input.name, value: label.input.name }]
    : [])
].toSorted((a, b) => a.label.localeCompare(b.label)));

const selection = computed<(number | string)[]>({
  get: () => model.value
    .map((label) => label.kind === "existing" ? label.id : label.input.name)
    .toSorted((a, b) => (items.value.find((item) => item.value === a)?.label ?? "")
      .localeCompare(items.value.find((item) => item.value === b)?.label ?? "")),
  set: (values) => {
    model.value = values.map((value): LabelRef => typeof value === "number"
      ? { kind: "existing", id: value }
      : { kind: "new", input: { name: value } });
    void emitFormInput();
    emitFormChange();
  }
});
const newLabelValidation = computed(() => labelUpsertInputGeneratedSchema.safeParse({ name: searchTerm.value.trim() }));
const canCreate = computed(() => newLabelValidation.value.success
  && !items.value.some((item) => item.label.toLowerCase() === searchTerm.value.trim().toLowerCase()));

function onCreate(name: string) {
  const parsed = labelUpsertInputGeneratedSchema.safeParse({ name: name.trim() });
  if (!parsed.success) return;
  const existing = items.value.find((item) => item.label.toLowerCase() === parsed.data.name.toLowerCase());
  const value = existing?.value ?? parsed.data.name;
  selection.value = [...new Set([...selection.value, value])];
  searchTerm.value = "";
}

// After a successful save or a catalogue refresh, staged names can become persisted IDs.
watch(() => labelsQuery.data, (labels) => {
  if (!labels) return;
  model.value = model.value.map((label) => {
    if (label.kind === "existing") return label;
    const existing = labels.find((item) => item.name.toLowerCase() === label.input.name.toLowerCase());
    return existing ? { kind: "existing", id: existing.id } : label;
  });
});
</script>
