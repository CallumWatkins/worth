<template>
  <UModal
    v-model:open="open"
    title="Delete account"
    :dismissible="false"
    :close="false"
  >
    <template #body>
      <div class="space-y-4">
        <UAlert
          color="error"
          variant="soft"
          title="This action is permanent"
          description="Deleting this account will also delete all of its balance snapshots. This action cannot be undone."
        />

        <UAlert
          v-if="submitError"
          color="error"
          variant="subtle"
          :title="submitError"
        />

        <UAlert
          v-if="deletePreviewQuery.isError"
          color="error"
          variant="subtle"
          :title="deletePreviewQuery.error.message ?? 'Failed to load delete preview'"
        />

        <template v-if="deletePreviewQuery.isSuccess">
          <div class="space-y-1.5 text-sm text-toned">
            <div class="font-medium text-highlighted">
              You are about to delete:
            </div>
            <ul class="list-disc list-inside space-y-1">
              <li>
                Account: <span class="text-highlighted">{{ deletePreviewQuery.data.name }}</span>
              </li>
              <li>
                Snapshots: <span class="text-highlighted">{{ deletePreviewQuery.data.snapshot_count }}</span>
              </li>
            </ul>
            <div>
              The institution <span class="text-highlighted">{{ deletePreviewQuery.data.institution_name }}</span> will not be deleted.
            </div>
          </div>

          <UFormField>
            <template #label>
              Type <span class="text-highlighted">{{ CONFIRM_PHRASE }}</span> to confirm
            </template>
            <UInput
              v-model="confirmationInput"
              :placeholder="CONFIRM_PHRASE"
              class="w-full"
              autofocus
            />
          </UFormField>
        </template>

        <div class="flex justify-end gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :disabled="deleteAccount.isPending.value"
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
            Delete account permanently
          </UButton>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";

const props = defineProps<{
  accountId: number | null
}>();

const CONFIRM_PHRASE = "delete";

const open = defineModel<boolean>("open", { required: true });
const api = useApi();
const { deleteAccount } = useAccountMutations();

const confirmationInput = ref("");
const submitError = ref<string | null>(null);

const deletePreviewQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.deletePreview(props.accountId!)),
  enabled: computed(() => open.value && props.accountId !== null),
  queryFn: () => api.accountsDeletePreview(props.accountId!)
}));

const canDelete = computed(() => {
  if (deleteAccount.isPending.value) return false;
  if (!deletePreviewQuery.isSuccess) return false;
  return confirmationInput.value.trim().toLowerCase() === CONFIRM_PHRASE;
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  confirmationInput.value = "";
  submitError.value = null;
});

async function onDelete() {
  if (props.accountId === null || !canDelete.value) return;
  submitError.value = null;

  try {
    await deleteAccount.mutateAsync(props.accountId);
    open.value = false;
    await navigateTo({ name: "accounts" });
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Failed to delete account";
  }
}
</script>
