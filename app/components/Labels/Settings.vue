<template>
  <UPageCard id="account-labels" title="Account labels">
    <div v-if="labelsQuery.data?.length" class="flex items-center gap-3">
      <UInput v-model="searchTerm" icon="i-lucide-search" placeholder="Search labels" aria-label="Search labels" class="flex-1" />
      <UButton icon="i-lucide-plus" @click="editingLabel = null; editOpen = true">
        Create label
      </UButton>
    </div>
    <UAlert v-if="labelsQuery.isError" color="error" variant="subtle" title="Could not load labels">
      <template #actions>
        <UButton color="neutral" variant="subtle" :loading="labelsQuery.isFetching" @click="labelsQuery.refetch()">
          Retry
        </UButton>
      </template>
    </UAlert>
    <p v-else-if="labelsQuery.isPending" class="text-sm text-muted" role="status">
      Loading labels…
    </p>
    <div v-else-if="!labelsQuery.data?.length" class="flex items-center gap-3">
      <UAvatar icon="i-lucide-tag" size="lg" aria-hidden="true" />
      <div class="flex-1 min-w-0" :class="[settingsRowsClass]">
        <UFormField
          label="No labels yet"
          description="Create a label to start organizing your accounts."
          orientation="horizontal"
          :ui="settingsFieldUi"
        >
          <UButton icon="i-lucide-plus" @click="editingLabel = null; editOpen = true">
            Create label
          </UButton>
        </UFormField>
      </div>
    </div>
    <UTable v-else-if="visibleLabels.length" :data="visibleLabels" :columns="columns" class="max-h-80" :ui="{ base: 'table-auto w-full', thead: 'hidden' }">
      <template #name-cell="{ row }">
        <UBadge color="neutral" variant="soft" class="max-w-full">
          {{ row.original.name }}
        </UBadge>
      </template>
      <template #description-cell="{ row }">
        <span class="whitespace-pre-wrap wrap-anywhere text-muted">{{ row.original.description ?? '—' }}</span>
      </template>
      <template #account_count-cell="{ row }">
        <UTooltip :text="`${row.original.account_count} ${row.original.account_count === 1 ? 'account' : 'accounts'}`">
          <ULink
            v-show="row.original.account_count > 0"
            :to="{ name: 'accounts', query: { label: row.original.id } }"
            class="inline-flex items-center gap-1 tabular-nums"
            :aria-label="`View accounts using ${row.original.name}`"
          >
            <UIcon name="i-lucide-wallet" class="size-4" aria-hidden="true" />
            {{ row.original.account_count }}
            <span class="sr-only">{{ row.original.account_count === 1 ? 'account' : 'accounts' }}</span>
          </ULink>
        </UTooltip>
      </template>
      <template #actions-cell="{ row }">
        <UDropdownMenu
          :items="[
            { label: 'Edit', icon: 'i-lucide-pencil', onSelect: () => { editingLabel = row.original; editOpen = true; } },
            { label: 'Delete', icon: 'i-lucide-trash-2', color: 'error', onSelect: () => { deletingLabel = row.original; deleteOpen = true; } }
          ]"
          :content="{ align: 'end' }"
        >
          <UButton icon="i-lucide-ellipsis-vertical" color="neutral" variant="ghost" :aria-label="`Actions for ${row.original.name}`" />
        </UDropdownMenu>
      </template>
    </UTable>
    <p v-else class="text-sm text-muted">
      No labels match your search.
    </p>
    <LabelsEditDialog v-model:open="editOpen" :label="editingLabel" />
    <LabelsDeleteDialog v-model:open="deleteOpen" :label="deletingLabel" />
  </UPageCard>
</template>

<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { LabelDto, LabelSummaryDto } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

const api = useApi();
const labelsQuery = proxyRefs(useQuery({ queryKey: queryKeys.labels.list(), queryFn: api.labelsList }));
const searchTerm = ref("");
watch(() => labelsQuery.data?.length, (count) => {
  if (count === 0) searchTerm.value = "";
});
const editOpen = ref(false);
const deleteOpen = ref(false);
const editingLabel = ref<LabelDto | null>(null);
const deletingLabel = ref<LabelSummaryDto | null>(null);
const columns: TableColumn<LabelSummaryDto>[] = [
  { accessorKey: "name", header: "Name", meta: { class: { td: "w-px whitespace-nowrap" } } },
  { accessorKey: "description", header: "Description", meta: { class: { td: "w-full whitespace-normal" } } },
  { accessorKey: "account_count", header: "Accounts", meta: { class: { td: "w-20 text-right" } } },
  { id: "actions", header: "", meta: { class: { td: "w-14 text-right" } } }
];
const visibleLabels = computed(() => (labelsQuery.data ?? [])
  .filter((label) => [label.name, label.description ?? ""].some((value) => value.toLowerCase().includes(searchTerm.value.trim().toLowerCase())))
  .toSorted((a, b) => a.name.localeCompare(b.name)));
</script>
