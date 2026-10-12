<template>
  <UCollapsible v-model:open="open" :unmount-on-hide="false" class="space-y-3">
    <UButton
      label="Account details"
      color="neutral"
      variant="ghost"
      class="-ms-2.5"
    >
      <template #trailing>
        <span v-if="!model.length" class="text-muted font-normal">Optional</span>
        <UIcon :name="open ? 'i-lucide-chevron-up' : 'i-lucide-chevron-down'" />
      </template>
    </UButton>

    <template #content>
      <div class="@container space-y-3">
        <div class="grid gap-2" :class="columnsClass">
          <div v-if="model.length" class="col-span-full grid grid-cols-subgrid text-sm font-medium" aria-hidden="true">
            <span :class="{ 'col-start-2': model.length > 1 }">Identifier</span>
            <span>Value</span>
          </div>
          <div ref="rowsElement" class="col-span-full space-y-2">
            <div v-for="({ identifier, key }, index) in rows" :key="key" class="grid items-start gap-2" :class="columnsClass">
              <UDropdownMenu
                v-if="model.length > 1"
                :items="[
                  { label: 'Move up', icon: 'i-lucide-arrow-up', disabled: index === 0, onSelect: () => move(index, index - 1) },
                  { label: 'Move down', icon: 'i-lucide-arrow-down', disabled: index === model.length - 1, onSelect: () => move(index, index + 1) }
                ]"
                :content="popupContent"
                :disabled="disabled"
              >
                <UButton
                  data-drag-handle
                  icon="i-lucide-grip-vertical"
                  color="neutral"
                  variant="link"
                  class="px-0 cursor-grab active:cursor-grabbing"
                  :disabled="disabled"
                  :aria-label="`Reorder identifier ${index + 1}`"
                  aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown"
                  @keydown.alt.up.capture.stop.prevent="move(index, index - 1)"
                  @keydown.alt.down.capture.stop.prevent="move(index, index + 1)"
                />
              </UDropdownMenu>
              <UFormField :name="`identifiers.${index}.label`" class="min-w-0">
                <UInputMenu
                  :id="`${id}-${index}-label`"
                  :model-value="identifier.label"
                  :items="identifier.label.trim() && !suggestions.includes(identifier.label) ? [identifier.label, ...suggestions] : suggestions"
                  :ignore-filter="filteringIdentifierKey !== key"
                  mode="autocomplete"
                  open-on-focus
                  aria-label="Identifier"
                  placeholder="e.g. account number"
                  autocomplete="off"
                  :spellcheck="false"
                  class="w-full"
                  @input="filteringIdentifierKey = key"
                  @update:open="!$event && (filteringIdentifierKey = null)"
                  @update:model-value="update(index, 'label', $event)"
                >
                  <template #item-trailing="{ item, ui }">
                    <UIcon v-if="item === identifier.label" name="i-lucide-check" :class="ui.itemTrailingIcon()" />
                  </template>
                </UInputMenu>
              </UFormField>
              <UFormField :name="`identifiers.${index}.value`" class="min-w-0">
                <UInput
                  :id="`${id}-${index}-value`"
                  :model-value="identifier.value"
                  aria-label="Value"
                  autocomplete="off"
                  :spellcheck="false"
                  class="w-full"
                  @update:model-value="update(index, 'value', String($event))"
                />
              </UFormField>
              <UButton
                icon="i-lucide-x"
                color="neutral"
                variant="ghost"
                :disabled="disabled"
                :aria-label="`Remove identifier ${index + 1}`"
                @click="remove(index)"
              />
            </div>
          </div>
        </div>
        <span class="sr-only" role="status">{{ announcement }}</span>
        <UButton
          label="Add identifier"
          icon="i-lucide-plus"
          color="neutral"
          variant="subtle"
          :disabled="disabled"
          @click="add"
        />
      </div>
    </template>
  </UCollapsible>
</template>

<script setup lang="ts">
import type { AccountIdentifierInput } from "~/generated/bindings";
import { formErrorsInjectionKey, useFormField } from "@nuxt/ui/composables/useFormField";
import { useSortable } from "@vueuse/integrations/useSortable";

const props = defineProps<{ expanded?: boolean }>();
const emit = defineEmits<{ remove: [], reorder: [] }>();
const model = defineModel<AccountIdentifierInput[]>({ default: () => [] });
const { disabled, emitFormInput, emitFormChange } = useFormField({});
const errors = inject(formErrorsInjectionKey, ref([]));
const open = ref(props.expanded || model.value.length > 0);
const rowKeys = ref<number[]>([]);
const filteringIdentifierKey = ref<number | null>(null);
// Position-based input IDs keep Nuxt UI's registered error targets valid after moves.
const id = useId();
let nextRowKey = 0;
const rowsElement = useTemplateRef<HTMLDivElement>("rowsElement");
const popupContent = useAppPopupContent({ align: "start" });
const reducedMotion = usePreferredReducedMotion();
const announcement = ref("");
const columnsClass = computed(() => model.value.length > 1
  ? "grid-cols-[1.25rem_minmax(0,1fr)_minmax(0,1fr)_2rem] @xl:grid-cols-[1.25rem_18rem_minmax(0,1fr)_2rem]"
  : "grid-cols-[minmax(0,1fr)_minmax(0,1fr)_2rem] @xl:grid-cols-[18rem_minmax(0,1fr)_2rem]");
const rows = computed({
  get: () => model.value.map((identifier, index) => ({ identifier, key: rowKeys.value[index]! })),
  set: (value) => {
    rowKeys.value = value.map((row) => row.key);
    model.value = value.map((row) => row.identifier);
    void emitFormInput();
    emitFormChange();
    emit("reorder");
  }
});
const { option } = useSortable(rowsElement, rows, {
  handle: "[data-drag-handle]",
  direction: "vertical",
  // Tauri handles native HTML drag events; use Sortable's pointer-based dragging.
  forceFallback: true,
  fallbackOnBody: true,
  fallbackTolerance: 4,
  animation: reducedMotion.value === "reduce" ? 0 : 180,
  disabled: disabled.value,
  watchElement: true,
  ghostClass: "opacity-40",
  onChoose: ({ item }) => {
    // The body-level preview cannot inherit the editor's container query width.
    item.style.gridTemplateColumns = getComputedStyle(item).gridTemplateColumns;
  },
  onUnchoose: ({ item }) => {
    item.style.removeProperty("grid-template-columns");
  }
});
watch(disabled, (value) => option("disabled", !!value));
watch(reducedMotion, (value) => option("animation", value === "reduce" ? 0 : 180));

// Suggestions are labels, not a closed catalogue of formats or countries.
const suggestions = [
  "Account number",
  "Sort code",
  "IBAN",
  "SWIFT/BIC",
  "Routing number",
  "BSB",
  "Transit number",
  "Institution number",
  "Branch code",
  "Bank code",
  "IFSC",
  "CLABE",
  "Account holder",
  "Roll number",
  "Customer number",
  "Member number",
  "Policy number",
  "Plan number",
  "Reference"
];

watch(() => model.value.length, (length) => {
  rowKeys.value = Array.from({ length }, (_, index) => rowKeys.value[index] ?? nextRowKey++);
}, { immediate: true });
watch(() => props.expanded, (expanded) => {
  if (expanded) open.value = true;
});
watch(errors, (issues) => {
  if (issues.some((issue) => issue.name === "identifiers" || issue.name?.startsWith("identifiers."))) {
    open.value = true;
  }
});

function update(index: number, field: keyof AccountIdentifierInput, value: string) {
  model.value = model.value.map((identifier, current) => current === index ? { ...identifier, [field]: value } : identifier);
}

function add() {
  model.value = [...model.value, { label: "", value: "" }];
  void emitFormInput();
  emitFormChange();
}

async function move(from: number, to: number) {
  if (disabled.value || to < 0 || to >= model.value.length) return;
  announcement.value = `${model.value[from]!.label || "Identifier"} moved to position ${to + 1} of ${model.value.length}`;
  const reordered = [...rows.value];
  const [row] = reordered.splice(from, 1);
  reordered.splice(to, 0, row!);
  rows.value = reordered;
  await nextTick();
  rowsElement.value?.children[to]?.querySelector<HTMLButtonElement>("[data-drag-handle]")?.focus();
}

function remove(index: number) {
  rowKeys.value.splice(index, 1);
  model.value = model.value.filter((_, current) => current !== index);
  void emitFormInput();
  emitFormChange();
  // The parent clears errors whose paths used the previous row indices.
  emit("remove");
}
</script>
