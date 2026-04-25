<template>
  <UModal
    v-model:open="open"
    title="Delete institution"
    :dismissible="true"
    :close="false"
  >
    <template #body>
      <UForm
        :state="confirmationState"
        class="space-y-4"
        @submit="onDelete"
      >
        <UAlert
          color="error"
          variant="soft"
          title="This action is permanent"
          description="Deleting this institution will also delete all of its accounts and their balance snapshots. This action cannot be undone."
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
          :title="deletePreviewQuery.error.message"
        />

        <template v-if="deletePreviewQuery.isSuccess">
          <div class="space-y-1.5 text-sm text-toned">
            <div class="font-medium text-highlighted">
              You are about to delete:
            </div>
            <ul class="list-disc list-inside space-y-1">
              <li>
                Institution: <span class="text-highlighted">{{ deletePreviewQuery.data.institution.name }}</span>
              </li>
              <li>
                Accounts: <span class="text-highlighted">{{ deletePreviewQuery.data.accounts.length }}</span>
              </li>
              <li>
                Snapshots: <span class="text-highlighted">{{ deletePreviewQuery.data.total_snapshots }}</span>
              </li>
            </ul>
          </div>

          <div v-if="deletePreviewQuery.data.accounts.length > 0" class="rounded-md border border-default">
            <div class="px-3 py-2 text-sm font-medium bg-elevated/50">
              Accounts that will be deleted
            </div>
            <div class="divide-y divide-default">
              <div
                v-for="account in deletePreviewQuery.data.accounts"
                :key="account.id"
                class="px-3 py-2 flex items-start justify-between gap-3 text-sm"
              >
                <span class="text-highlighted wrap-anywhere">{{ account.name }}</span>
                <span class="text-toned shrink-0">{{ formatCount(account.snapshot_count, "snapshot") }}</span>
              </div>
            </div>
          </div>

          <UFormField name="confirmationInput">
            <template #label>
              Type <span class="text-highlighted">{{ CONFIRM_PHRASE }}</span> to confirm
            </template>
            <UInput
              v-model="confirmationState.confirmationInput"
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
            type="button"
            :disabled="deleteInstitution.isPending"
            @click="open = false"
          >
            Cancel
          </UButton>
          <UButton
            color="error"
            type="submit"
            :disabled="!canDelete"
            loading-auto
          >
            Delete institution permanently
          </UButton>
        </div>
      </UForm>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";

const props = defineProps<{
  institutionId: number | null
}>();

const CONFIRM_PHRASE = "delete";

const open = defineModel<boolean>("open", { required: true });
const api = useApi();
const { deleteInstitution } = useInstitutionMutations();

const confirmationState = reactive({ confirmationInput: "" });
const submitError = ref<string | null>(null);

const deletePreviewQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.deletePreview(props.institutionId!)),
  enabled: computed(() => open.value && props.institutionId !== null),
  queryFn: async () => api.institutionsDeletePreview(props.institutionId!)
}));

const canDelete = computed(() => {
  if (deleteInstitution.isPending) return false;
  if (!deletePreviewQuery.isSuccess) return false;
  return confirmationState.confirmationInput.trim().toLowerCase() === CONFIRM_PHRASE;
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  confirmationState.confirmationInput = "";
  submitError.value = null;
});

async function onDelete() {
  if (props.institutionId === null || !canDelete.value) return;
  submitError.value = null;

  try {
    await deleteInstitution.mutateAsync(props.institutionId);
    open.value = false;
    await navigateTo({ name: "institutions" });
  } catch (error) {
    submitError.value = error instanceof Error ? error.message : "Failed to delete institution";
  }
}

function formatCount(value: number, noun: string) {
  return `${value} ${noun}${value === 1 ? "" : "s"}`;
}
</script>
