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

    <UFormField
      v-if="hasIsoDateTimeFormat"
      label="Timestamp date"
      description="How to turn timestamps into Worth snapshot dates."
    >
      <USelect
        :model-value="modelValue.source.timestamp_date_policy"
        :items="timestampDatePolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('timestamp_date_policy', $event as CsvSnapshotImportTimestampDatePolicy)"
      />
    </UFormField>

    <UFormField
      v-if="usesMissingTimezonePolicy"
      label="Missing timezone"
      description="How to interpret ISO timestamps that do not include Z or a UTC offset."
    >
      <USelect
        :model-value="modelValue.source.timestamp_missing_timezone_policy"
        :items="missingTimezonePolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('timestamp_missing_timezone_policy', $event as CsvSnapshotImportMissingTimezonePolicy)"
      />
    </UFormField>

    <div
      v-if="usesMissingTimezonePolicy && modelValue.source.timestamp_missing_timezone_policy === 'timezone'"
      class="ml-3 border-l border-default pl-4"
    >
      <UFormField
        label="Timezone"
        description="Choose the timezone the timestamp values were exported in."
      >
        <USelectMenu
          :model-value="modelValue.source.timestamp_missing_timezone"
          :items="timezoneItems"
          value-key="value"
          :disabled="busy"
          class="w-full"
          placeholder="Choose timezone"
          :ui="{
            base: 'ps-[58px]',
            item: 'items-center'
          }"
          @update:model-value="updateSourceOption('timestamp_missing_timezone', String($event ?? ''))"
        >
          <template #leading>
            <span v-if="selectedTimezoneOffset" class="text-xs tabular-nums text-muted">
              {{ selectedTimezoneOffset }}
            </span>
          </template>
          <template #item-leading="{ item }">
            <span class="text-xs tabular-nums text-muted">
              {{ item.offset }}
            </span>
          </template>
        </USelectMenu>
      </UFormField>
    </div>

    <UFormField label="Balance format">
      <USelect
        :model-value="modelValue.source.balance_format"
        :items="balanceFormatItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('balance_format', $event as CsvSnapshotImportBalanceFormat)"
      />
    </UFormField>

    <UFormField label="Blank amount cell" description="What to do when an imported row has a date but no amount.">
      <USelect
        :model-value="modelValue.source.blank_amount_policy"
        :items="blankAmountPolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updateSourceOption('blank_amount_policy', $event as CsvSnapshotImportBlankAmountPolicy)"
      />
    </UFormField>

    <UFormField label="Multiple rows for same date" description="What to do when multiple imported rows resolve to the same snapshot date.">
      <USelect
        :model-value="modelValue.duplicate_date_policy"
        :items="duplicateDatePolicyItems"
        :disabled="busy"
        class="w-full"
        @update:model-value="updatePolicyOption('duplicate_date_policy', $event as SnapshotImportDuplicateDatePolicy)"
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
  CsvSnapshotImportBlankAmountPolicy,
  CsvSnapshotImportDateFormat,
  CsvSnapshotImportMissingTimezonePolicy,
  CsvSnapshotImportOptionsInput,
  CsvSnapshotImportTimestampDatePolicy,
  SnapshotImportDuplicateDatePolicy,
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
  { label: "YYYY/MM/DD", value: "yyyy_mm_dd_slash" },
  { label: "YYYY-MM-DDThh:mm:ss (ISO 8601)", value: "iso_8601_date_time" }
];

const timestampDatePolicyItems: Array<{ label: string, value: CsvSnapshotImportTimestampDatePolicy }> = [
  { label: "Use date as written", value: "date_as_written" },
  { label: "Convert to my local date", value: "convert_to_local" },
  { label: "Convert to UTC", value: "convert_to_utc" }
];

const missingTimezonePolicyItems: Array<{ label: string, value: CsvSnapshotImportMissingTimezonePolicy }> = [
  { label: "Treat as local time", value: "local" },
  { label: "Treat as UTC", value: "utc" },
  { label: "Choose timezone", value: "timezone" }
];

const timezoneItems = getTimezoneItems();
const selectedTimezoneOffset = computed(() => timezoneItems.find((item) => item.value === props.modelValue.source.timestamp_missing_timezone)?.offset);

const balanceFormatItems: Array<{ label: string, value: CsvSnapshotImportBalanceFormat }> = [
  { label: "1,234.56", value: "thousands_comma_decimal_dot" },
  { label: "1.234,56", value: "thousands_dot_decimal_comma" }
];

const blankAmountPolicyItems: Array<{ label: string, value: CsvSnapshotImportBlankAmountPolicy }> = [
  { label: "Skip row", value: "skip" },
  { label: "Treat as zero", value: "zero" },
  { label: "Treat as error", value: "error" }
];

const duplicateDatePolicyItems: Array<{ label: string, value: SnapshotImportDuplicateDatePolicy }> = [
  { label: "Use first row for each date", value: "keep_first" },
  { label: "Use last row for each date", value: "keep_last" },
  { label: "Treat as error", value: "error" }
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

const hasIsoDateTimeFormat = computed(() => props.modelValue.source.date_format === "iso_8601_date_time");
const usesMissingTimezonePolicy = computed(() => hasIsoDateTimeFormat.value && props.modelValue.source.timestamp_date_policy !== "date_as_written");

function getTimezoneItems() {
  const now = new Date();
  const offsetPattern = /^GMT([+-])(\d{1,2})(?::(\d{2}))?$/;

  return Intl.supportedValuesOf("timeZone").map((timezone) => {
    const offset = new Intl.DateTimeFormat("en-US", {
      timeZone: timezone,
      timeZoneName: "shortOffset"
    }).formatToParts(now).find((part) => part.type === "timeZoneName")?.value;
    const match = offset?.match(offsetPattern);

    return {
      label: timezone,
      value: timezone,
      offset: match == null ? "+00:00" : `${match[1]}${match[2]!.padStart(2, "0")}:${match[3] ?? "00"}`
    };
  });
}

function updateSourceOption<K extends keyof CsvSnapshotImportOptionsInput>(key: K, value: CsvSnapshotImportOptionsInput[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    source: {
      ...props.modelValue.source,
      [key]: value
    }
  });
}

function updatePolicyOption<K extends "duplicate_date_policy" | "existing_date_policy" | "unchanged_value_policy">(key: K, value: SnapshotImportOptionsInput[K]) {
  emit("update:modelValue", {
    ...props.modelValue,
    [key]: value
  });
}
</script>
