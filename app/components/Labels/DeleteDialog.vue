<template>
  <UModal
    v-model:open="open"
    title="Delete label"
    :dismissible="!deleteLabel.isPending"
    :close="!deleteLabel.isPending"
    :ui="{ footer: 'justify-end' }"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          color="error"
          variant="soft"
          title="This action is permanent"
          description="Deleting this label will remove it from all accounts. This action cannot be undone."
        />
        <p class="text-sm text-muted">
          <UBadge color="neutral" variant="soft">
            {{ label?.name }}
          </UBadge>
          {{ description }}
        </p>
        <UAlert v-if="accountsQuery.isError" color="error" variant="subtle" title="Could not load affected accounts">
          <template #actions>
            <UButton color="neutral" variant="subtle" :loading="accountsQuery.isFetching" @click="accountsQuery.refetch()">
              Retry
            </UButton>
          </template>
        </UAlert>
        <p v-else-if="accountsQuery.isPending" class="text-sm text-muted" role="status">
          Loading affected accounts…
        </p>
        <div v-else-if="affectedAccounts.length > 0" class="rounded-md border border-default">
          <div class="px-3 py-2 text-sm font-medium bg-elevated/50">
            Accounts using this label
          </div>
          <div class="divide-y divide-default">
            <div
              v-for="account in affectedAccounts"
              :key="account.id"
              class="px-3 py-2 flex items-start justify-between gap-3 text-sm"
            >
              <span class="text-highlighted wrap-anywhere">{{ account.name }}</span>
              <span class="text-toned text-right wrap-anywhere">{{ account.institution.name }}</span>
            </div>
          </div>
        </div>
        <UAlert v-if="submitError" color="error" variant="subtle" :title="submitError" />
      </div>
    </template>
    <template #footer>
      <UButton color="neutral" variant="ghost" :disabled="deleteLabel.isPending" @click="open = false">
        Cancel
      </UButton>
      <UButton color="error" :loading="deleteLabel.isPending" :disabled="!accountsQuery.isSuccess" @click="onDelete">
        Delete label
      </UButton>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { LabelSummaryDto } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

const props = defineProps<{ label: LabelSummaryDto | null }>();
const open = defineModel<boolean>("open", { required: true });
const api = useApi();
const accountsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.accounts.list(),
  queryFn: api.accountsList,
  enabled: computed(() => open.value && props.label !== null)
}));
const affectedAccounts = computed(() => (accountsQuery.data ?? [])
  .filter((account) => account.labels.some((label) => label.id === props.label?.id))
  .toSorted((a, b) => a.name.localeCompare(b.name)));
const { deleteLabel } = useLabelMutations();
const submitError = ref<string | null>(null);
const description = computed(() => {
  const count = accountsQuery.isSuccess ? affectedAccounts.value.length : props.label?.account_count ?? 0;
  return count === 0
    ? "is not assigned to any accounts."
    : `will be removed from ${count} ${count === 1 ? "account" : "accounts"}.`;
});

useNavigationLayer({
  id: "label-delete-dialog",
  open,
  pending: computed(() => deleteLabel.isPending),
  close: () => {
    open.value = false;
  }
});

watch(open, () => {
  submitError.value = null;
});

async function onDelete() {
  if (!props.label) return;
  submitError.value = null;
  try {
    await deleteLabel.mutateAsync(props.label.id);
    open.value = false;
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Could not delete label";
  }
}
</script>
