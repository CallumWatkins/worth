<template>
  <UModal
    v-model:open="open"
    :title="snapshots.length === 1 ? 'Delete snapshot' : 'Delete snapshots'"
    :dismissible="false"
    :close="false"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          color="error"
          variant="soft"
          title="This action is permanent"
          :description="snapshots.length === 1 ? 'Deleting this balance snapshot cannot be undone.' : 'Deleting these balance snapshots cannot be undone.'"
        />

        <UAlert
          v-if="submitError"
          color="error"
          variant="subtle"
          :title="submitError"
        />

        <div v-if="snapshots.length" class="space-y-2 text-sm text-toned">
          <div class="font-medium text-highlighted">
            You are about to delete {{ snapshots.length }} snapshot{{ snapshots.length === 1 ? '' : 's' }}:
          </div>
          <ul class="space-y-1">
            <li v-for="snapshot in snapshots.slice(0, 5)" :key="snapshot.id" class="flex items-center justify-between gap-3">
              <span>{{ formatShortDate(snapshot.date) }}</span>
              <span class="text-highlighted">{{ formatCurrencyMinor(snapshot.balance_minor, currencyCode) }}</span>
            </li>
          </ul>
          <div v-if="snapshots.length > 5" class="text-muted">
            And {{ snapshots.length - 5 }} more snapshot{{ snapshots.length - 5 === 1 ? '' : 's' }}.
          </div>
        </div>

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :disabled="deleteSnapshots.isPending"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            color="error"
            :disabled="!canDelete"
            loading-auto
            @click="onDelete"
          >
            Delete snapshot{{ snapshots.length === 1 ? '' : 's' }}
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { AccountBalanceSnapshotDto, CurrencyCode } from "~/generated/bindings";

const props = defineProps<{
  accountId: number | null
  currencyCode: CurrencyCode
  snapshots: AccountBalanceSnapshotDto[]
}>();

const open = defineModel<boolean>("open", { required: true });

const { formatCurrencyMinor, formatShortDate } = useLocaleFormatters();
const { deleteSnapshots } = useAccountSnapshotMutations();

const submitError = ref<string | null>(null);

const canDelete = computed(() => {
  return props.accountId != null && props.snapshots.length > 0 && !deleteSnapshots.isPending;
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  submitError.value = null;
});

async function onDelete() {
  if (props.accountId == null || !props.snapshots.length) return;
  submitError.value = null;

  try {
    await deleteSnapshots.mutateAsync({
      accountId: props.accountId,
      input: {
        snapshot_ids: props.snapshots.map((snapshot) => snapshot.id)
      }
    });
    open.value = false;
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Failed to delete snapshots";
  }
}
</script>
