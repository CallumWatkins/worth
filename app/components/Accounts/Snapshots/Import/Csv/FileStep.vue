<template>
  <div class="space-y-8">
    <div class="space-y-4">
      <UFileUpload
        :model-value="selectedFile"
        accept=".csv,text/csv"
        label="Drop or click to select a CSV"
        description="Import a local CSV file."
        :disabled="busy"
        layout="list"
        position="inside"
        @update:model-value="$emit('update:selectedFile', $event ?? null)"
      />

      <UCheckbox
        :model-value="hasHeaderRow"
        label="First row contains column names"
        description="Turn this off if the file starts with data instead of headers."
        :disabled="busy"
        @update:model-value="$emit('update:hasHeaderRow', Boolean($event))"
      />
    </div>

    <div v-if="inspection" class="space-y-3">
      <template v-if="hasSampleRows">
        <div class="flex justify-between">
          <span class="text-muted text-sm">
            The selected file contains {{ inspection.total_rows }} row{{ inspection.total_rows === 1 ? '' : 's' }} to import.
          </span>
          <UBadge label="Sample" variant="soft" color="neutral" />
        </div>

        <UTable
          :data="inspection.sample_rows"
          :columns="sampleColumns"
          :ui="{ th: 'py-2', td: 'py-2' }"
          class="overflow-auto rounded-lg border border-default"
        />
      </template>

      <UAlert
        v-else
        color="error"
        variant="subtle"
        title="No import rows found"
        description="Choose a CSV file that contains at least one row to import."
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { TableColumn } from "@nuxt/ui";
import type { CsvSnapshotImportInspectionDto, CsvSnapshotImportSampleRowDto } from "~/generated/bindings";

const props = defineProps<{
  selectedFile: File | null
  hasHeaderRow: boolean
  inspection: CsvSnapshotImportInspectionDto | null
  busy: boolean
}>();

defineEmits<{
  "update:selectedFile": [File | null]
  "update:hasHeaderRow": [boolean]
}>();

const sampleColumns = computed<TableColumn<CsvSnapshotImportSampleRowDto>[]>(() => props.inspection?.columns.map((column, index) => ({
  id: `column-${column.index}`,
  header: column.name,
  accessorFn: (row) => {
    const value = row.values[index];
    return value == null || value === "" ? "—" : value;
  },
  meta: {
    class: {
      th: "whitespace-nowrap",
      td: "whitespace-nowrap"
    }
  }
})) ?? []);

const hasSampleRows = computed(() => (props.inspection?.sample_rows.length ?? 0) > 0);
</script>
