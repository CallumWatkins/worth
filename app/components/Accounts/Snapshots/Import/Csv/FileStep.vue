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
      <div class="text-sm text-muted">
        <span class="font-medium text-highlighted">{{ inspection.file_name }}</span>
        contains {{ inspection.total_rows }} import row{{ inspection.total_rows === 1 ? '' : 's' }}.
      </div>

      <UTable
        :data="inspection.sample_rows"
        :columns="sampleColumns"
        class="overflow-auto rounded-lg border border-default"
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
</script>
