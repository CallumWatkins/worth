<template>
  <UModal
    v-model:open="open"
    :dismissible="false"
    :title="dialogSnapshots.length === 1 ? 'Delete snapshot' : 'Delete snapshots'"
    :close="false"
    @after:leave="onAfterLeave"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          color="error"
          variant="soft"
          title="This action is permanent"
          :description="dialogSnapshots.length === 1 ? 'Deleting this balance snapshot cannot be undone.' : 'Deleting these balance snapshots cannot be undone.'"
        />

        <UAlert
          v-if="submitError"
          color="error"
          variant="subtle"
          :title="submitError"
        />

        <div class="space-y-1.5 text-sm text-toned">
          <div class="font-medium text-highlighted">
            You are about to delete:
          </div>
          <ul class="list-disc list-inside space-y-1">
            <li>
              Snapshots: <span class="text-highlighted">{{ dialogSnapshots.length }}</span>
            </li>
          </ul>
        </div>

        <div class="rounded-md border border-default">
          <div class="px-3 py-2 text-sm font-medium bg-elevated/50">
            Snapshots that will be deleted
          </div>
          <div class="divide-y divide-default">
            <div
              v-for="snapshot in visibleSnapshots"
              :key="snapshot.id"
              class="px-3 py-2 flex items-start justify-between gap-3 text-sm"
            >
              <span class="text-highlighted wrap-anywhere">{{ formatShortDate(snapshot.date) }}</span>
              <span class="text-highlighted">{{ formatCurrencyMinor(snapshot.balance_minor, currencyCode) }}</span>
            </div>
            <div v-if="dialogSnapshots.length > 5" class="px-3 py-2 flex items-start justify-between gap-3 text-sm">
              <span v-if="!showAllSnapshots">And {{ dialogSnapshots.length - 5 }} more snapshot{{ dialogSnapshots.length - 5 === 1 ? '' : 's' }}</span>
              <span v-else />
              <button type="button" class="text-highlighted cursor-pointer" @click="showAllSnapshots = !showAllSnapshots">
                {{ showAllSnapshots ? 'Show less' : 'Show all' }}
              </button>
            </div>
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
            Delete snapshot{{ dialogSnapshots.length === 1 ? '' : 's' }}
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
const dialogSnapshots = ref<AccountBalanceSnapshotDto[]>([]);
const showAllSnapshots = ref(false);

const visibleSnapshots = computed(() => {
  return showAllSnapshots.value ? dialogSnapshots.value : dialogSnapshots.value.slice(0, 5);
});

const canDelete = computed(() => {
  return props.accountId != null && dialogSnapshots.value.length > 0 && !deleteSnapshots.isPending;
});

watch(open, (isOpen) => {
  if (!isOpen) return;

  dialogSnapshots.value = [...props.snapshots];
  submitError.value = null;
  showAllSnapshots.value = false;
});

function onAfterLeave() {
  dialogSnapshots.value = [];
}

async function onDelete() {
  if (props.accountId == null || !dialogSnapshots.value.length) return;
  submitError.value = null;

  try {
    await deleteSnapshots.mutateAsync({
      accountId: props.accountId,
      input: {
        snapshot_ids: dialogSnapshots.value.map((snapshot) => snapshot.id)
      }
    });
    open.value = false;
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Failed to delete snapshots";
  }
}
</script>
