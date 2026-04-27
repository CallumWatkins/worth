<template>
  <div class="space-y-4">
    <UFormField label="Date column">
      <USelect
        :model-value="modelValue.source.date_column"
        :items="columnItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('date_column', String($event ?? ''))"
      />
    </UFormField>

    <UFormField label="Amount column">
      <USelect
        :model-value="modelValue.source.amount_column"
        :items="columnItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('amount_column', String($event ?? ''))"
      />
    </UFormField>

    <UFormField label="Date format">
      <USelect
        :model-value="modelValue.source.date_format"
        :items="dateFormatItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('date_format', $event as CsvSnapshotImportDateFormat)"
      />
    </UFormField>

    <UFormField label="Balance format">
      <USelect
        :model-value="modelValue.source.balance_format"
        :items="balanceFormatItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('balance_format', $event as CsvSnapshotImportBalanceFormat)"
      />
    </UFormField>

    <UFormField label="Existing date" description="What to do when a snapshot already exists for the imported date with a different balance.">
      <USelect
        :model-value="modelValue.existing_date_policy"
        :items="existingDatePolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updatePolicyOption('existing_date_policy', $event as SnapshotImportExistingDatePolicy)"
      />
    </UFormField>

    <UFormField label="Unchanged value" description="What to do when an imported balance is unchanged from the previous effective balance.">
      <USelect
        :model-value="modelValue.unchanged_value_policy"
        :items="unchangedValuePolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updatePolicyOption('unchanged_value_policy', $event as SnapshotImportUnchangedValuePolicy)"
      />
    </UFormField>
  </div>
</template>

<script lang="ts" setup>
import type {
  CsvSnapshotImportBalanceFormat,
  CsvSnapshotImportDateFormat,
  CsvSnapshotImportOptionsInput,
  SnapshotImportExistingDatePolicy,
  SnapshotImportOptionsInput,
  SnapshotImportUnchangedValuePolicy
} from "~/generated/bindings";

const props = defineProps<{
  modelValue: SnapshotImportOptionsInput
  columnItems: Array<{ label: string, value: string }>
  busy: boolean
}>();

const emit = defineEmits<{
  "update:modelValue": [SnapshotImportOptionsInput]
}>();

const dateFormatItems: Array<{ label: string, value: CsvSnapshotImportDateFormat }> = [
  { label: "YYYY-MM-DD", value: "yyyy_mm_dd" },
  { label: "DD/MM/YYYY", value: "dd_mm_yyyy_slash" },
  { label: "DD/MM/YY", value: "dd_mm_yy_slash" },
  { label: "MM/DD/YYYY", value: "mm_dd_yyyy_slash" },
  { label: "MM/DD/YY", value: "mm_dd_yy_slash" },
  { label: "DD-MM-YYYY", value: "dd_mm_yyyy_dash" },
  { label: "YYYY/MM/DD", value: "yyyy_mm_dd_slash" }
];

const balanceFormatItems: Array<{ label: string, value: CsvSnapshotImportBalanceFormat }> = [
  { label: "1,234.56", value: "thousands_comma_decimal_dot" },
  { label: "1.234,56", value: "thousands_dot_decimal_comma" }
];

const existingDatePolicyItems: Array<{ label: string, value: SnapshotImportExistingDatePolicy }> = [
  { label: "Overwrite existing snapshot", value: "overwrite" },
  { label: "Skip imported row", value: "skip" },
  { label: "Treat as error", value: "error" }
];

const unchangedValuePolicyItems: Array<{ label: string, value: SnapshotImportUnchangedValuePolicy }> = [
  { label: "Exclude unchanged row", value: "exclude" },
  { label: "Include unchanged row", value: "include" }
];

function updateSourceOption<K extends keyof CsvSnapshotImportOptionsInput>(key: K, value: CsvSnapshotImportOptionsInput[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    source: {
      ...props.modelValue.source,
      [key]: value
    }
  });
}

function updatePolicyOption<K extends "existing_date_policy" | "unchanged_value_policy">(key: K, value: SnapshotImportOptionsInput[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    [key]: value
  });
}
</script>
