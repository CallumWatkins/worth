<template>
  <div class="space-y-4">
    <div v-if="preview" class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-6">
        <div v-for="item in summaryItems" :key="item.label" class="rounded-lg border p-3" :class="item.class">
          <div class="text-xs uppercase tracking-wide text-muted">
            {{ item.label }}
          </div>
          <div class="mt-1 text-lg text-highlighted" :class="item.value > 0 ? 'font-semibold' : undefined">
            {{ item.value }}
          </div>
        </div>
      </div>

      <UAlert
        v-if="preview.summary.invalid_count > 0"
        color="error"
        variant="subtle"
        title="Some rows cannot be imported"
        :description="invalidPreviewDescription"
      />

      <UAlert
        v-else-if="warningCount > 0"
        color="warning"
        variant="subtle"
        title="Review warnings before importing"
        :description="`${warningCount} ${warningCount === 1 ? 'warning' : 'warnings'} found. Review the highlighted notes below before importing.`"
      />

      <UAlert
        v-else-if="preview.summary.skip_count === preview.summary.total_rows"
        color="neutral"
        variant="subtle"
        title="No changes to import"
        description="All rows will be skipped. Review the notes below to find out why."
      />

      <div class="relative rounded-lg after:pointer-events-none after:absolute after:inset-0 after:z-10 after:rounded-lg after:border after:border-default">
        <UTable
          :data="preview.rows"
          :columns="previewColumns"
          :ui="{
            root: 'min-w-0 overflow-x-hidden',
            base: 'w-full table-fixed'
          }"
          virtualize
          sticky
          class="max-h-[440px] overflow-y-auto"
        >
          <template #action-cell="{ row }">
            <UBadge :color="actionColor(row.original.action)" variant="subtle">
              {{ actionLabel(row.original.action) }}
            </UBadge>
          </template>

          <template #date-cell="{ row }">
            {{ row.original.date ? formatShortDate(row.original.date) : row.original.raw_date || '—' }}
          </template>

          <template #balance-cell="{ row }">
            {{ row.original.balance_minor == null ? (row.original.raw_amount || '—') : formatCurrencyMinor(row.original.balance_minor, currencyCode) }}
          </template>

          <template #notes-cell="{ row }">
            <div class="space-y-1">
              <div v-if="row.original.issues.length" class="space-y-1 text-error">
                <div v-for="issue in row.original.issues" :key="issue">
                  {{ issue }}
                </div>
              </div>

              <div v-if="row.original.warnings.length" class="space-y-1 text-warning">
                <div v-for="warning in row.original.warnings" :key="warning">
                  {{ warning }}
                </div>
              </div>

              <div v-if="row.original.action === 'overwrite' && row.original.existing_snapshot" class="text-toned">
                Overwrites the existing snapshot of {{ formatCurrencyMinor(row.original.existing_snapshot.balance_minor, currencyCode) }}.
              </div>

              <div v-if="row.original.action === 'skip_existing'" class="text-muted">
                Existing snapshot kept.
              </div>

              <div v-if="row.original.action === 'skip_duplicate'" class="text-muted">
                Another row for this date was selected.
              </div>

              <div v-if="row.original.action === 'skip_blank_amount'" class="text-muted">
                Blank amount row skipped.
              </div>

              <div
                v-if="row.original.action === 'skip_unchanged' && row.original.existing_snapshot != null && row.original.balance_minor === row.original.existing_snapshot.balance_minor"
                class="text-muted"
              >
                Same as existing snapshot.
              </div>

              <div v-else-if="row.original.action === 'skip_unchanged'" class="text-muted">
                Same as previous effective balance.
              </div>
            </div>
          </template>
        </UTable>
      </div>

      <div v-if="preview.summary.overwrite_count > 0" class="rounded-lg border border-warning/40 bg-warning/10 p-3">
        <UCheckbox
          :model-value="overwriteExistingConfirmed"
          :label="`I understand this import will overwrite ${preview.summary.overwrite_count} existing ${preview.summary.overwrite_count === 1 ? 'snapshot' : 'snapshots'}`"
          @update:model-value="emit('update:overwriteExistingConfirmed', Boolean($event))"
        />
      </div>
    </div>

    <div v-else class="rounded-lg border border-default p-6 text-center text-muted">
      {{ emptyPreviewDescription }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { TableColumn } from "@nuxt/ui";
import type {
  CurrencyCode,
  SnapshotImportPreviewAction,
  SnapshotImportPreviewDto,
  SnapshotImportPreviewRowDto
} from "~/generated/bindings";

const props = defineProps<{
  preview: SnapshotImportPreviewDto | null
  currencyCode: CurrencyCode
  overwriteExistingConfirmed: boolean
  invalidPreviewDescription: string
  emptyPreviewDescription: string
}>();

const emit = defineEmits<{
  "update:overwriteExistingConfirmed": [boolean]
}>();

const { formatCurrencyMinor, formatShortDate } = useLocaleFormatters();

const previewColumns: TableColumn<SnapshotImportPreviewRowDto>[] = [
  {
    accessorKey: "action",
    header: "Action",
    meta: {
      class: {
        th: "w-24",
        td: "w-24 whitespace-nowrap"
      }
    }
  },
  {
    accessorKey: "date",
    header: "Date",
    meta: {
      class: {
        th: "w-28",
        td: "w-28 whitespace-nowrap"
      }
    }
  },
  {
    id: "balance",
    accessorKey: "balance_minor",
    header: "Balance",
    meta: {
      class: {
        th: "w-32",
        td: "w-32 whitespace-nowrap"
      }
    }
  },
  {
    id: "notes",
    header: "Notes",
    meta: {
      class: {
        th: "w-full",
        td: "w-full min-w-0 whitespace-normal break-words align-top"
      }
    }
  }
];

const warningCount = computed(() => {
  return props.preview?.rows.reduce((count, row) => count + row.warnings.length, 0) ?? 0;
});

const summaryItems = computed(() => {
  const summary = props.preview?.summary;
  if (!summary) return [];
  return [
    { label: "Total", value: summary.total_rows, class: "border-default" },
    {
      label: "Create",
      value: summary.create_count,
      class: summary.create_count > 0 ? "border-default" : "border-default opacity-75"
    },
    {
      label: "Overwrite",
      value: summary.overwrite_count,
      class: summary.overwrite_count > 0 ? "border-warning/40 bg-warning/10" : "border-default opacity-75"
    },
    {
      label: "Skip",
      value: summary.skip_count,
      class: summary.skip_count > 0 ? "border-default" : "border-default opacity-75"
    },
    {
      label: "Invalid",
      value: summary.invalid_count,
      class: summary.invalid_count > 0 ? "border-error/40 bg-error/10" : "border-default opacity-75"
    },
    {
      label: "Warnings",
      value: warningCount.value,
      class: warningCount.value > 0 ? "border-warning/40 bg-warning/10" : "border-default opacity-75"
    }
  ];
});

function actionLabel(action: SnapshotImportPreviewAction) {
  switch (action) {
  case "create":
    return "Create";
  case "overwrite":
    return "Overwrite";
  case "skip_existing":
  case "skip_unchanged":
  case "skip_duplicate":
  case "skip_blank_amount":
    return "Skip";
  case "invalid":
    return "Invalid";
  }
}

function actionColor(action: SnapshotImportPreviewAction) {
  switch (action) {
  case "create":
    return "primary";
  case "overwrite":
    return "warning";
  case "invalid":
    return "error";
  case "skip_existing":
  case "skip_unchanged":
  case "skip_duplicate":
  case "skip_blank_amount":
    return "neutral";
  }
}
</script>
