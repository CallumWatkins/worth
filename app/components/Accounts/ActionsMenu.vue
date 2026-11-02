<template>
  <UDropdownMenu v-model:open="open" :items="items" :content="endPopupContent" aria-label="Account actions">
    <UButton
      icon="i-lucide-ellipsis-vertical"
      color="neutral"
      variant="ghost"
      :aria-label="`Actions for ${account.name}`"
    />
  </UDropdownMenu>
</template>

<script setup lang="ts">
import type { DropdownMenuItem } from "@nuxt/ui";
import type { AccountDto } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

const props = defineProps<{ account: AccountDto }>();
const emit = defineEmits<{ delete: [] }>();

const api = useApi();
const toast = useToast();
const endPopupContent = useAppPopupContent({ align: "end" });
const { setLabel, invalidateAccountWrites } = useAccountMutations();
const open = ref(false);
const pendingLabels = shallowRef<{ labelId: number, assigned: boolean }[]>([]);
const savedLabels = shallowRef<Record<number, boolean>>({});
let refreshing = false;
const labelsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.labels.list(),
  queryFn: api.labelsList,
  enabled: open
}));

async function refreshAccounts() {
  if (open.value || pendingLabels.value.length || Object.keys(savedLabels.value).length === 0 || refreshing) return;
  refreshing = true;
  const saved = savedLabels.value;
  try {
    await invalidateAccountWrites({ throwOnError: true });
    if (savedLabels.value === saved) savedLabels.value = {};
  } catch {
    toast.add({
      title: "Labels saved, but could not refresh accounts",
      color: "error",
      actions: [{
        label: "Retry",
        onClick: () => {
          void refreshAccounts();
        }
      }]
    });
  } finally {
    refreshing = false;
  }
  // A newer save may have completed while the previous refresh was in flight.
  if (savedLabels.value !== saved) void refreshAccounts();
}

watch(open, (isOpen) => {
  if (!isOpen) void refreshAccounts();
});

onScopeDispose(() => {
  // A save can finish after navigation has unmounted the menu.
  open.value = false;
  void refreshAccounts();
});

const items = computed<DropdownMenuItem[]>(() => [
  { type: "label", label: "Actions" },
  {
    label: "Labels",
    icon: "i-lucide-tag",
    content: { collisionBoundary: endPopupContent.value.collisionBoundary, collisionPadding: 8 },
    children: labelsQuery.isError
      ? [{
        label: "Could not load labels — retry",
        icon: "i-lucide-refresh-cw",
        onSelect: (event) => {
          event.preventDefault();
          void labelsQuery.refetch();
        }
      }]
      : labelsQuery.isPending
        ? [{ label: "Loading labels…", disabled: true }]
        : labelsQuery.data?.length
          ? labelsQuery.data.toSorted((a, b) => a.name.localeCompare(b.name)).map((label) => ({
            label: label.name,
            type: "checkbox",
            checked: pendingLabels.value.findLast((pending) => pending.labelId === label.id)?.assigned
              ?? savedLabels.value[label.id]
              ?? props.account.labels.some((assigned) => assigned.id === label.id),
            onSelect: (event) => event.preventDefault(),
            onUpdateChecked: async (assigned) => {
              const pending = { labelId: label.id, assigned };
              pendingLabels.value = [...pendingLabels.value, pending];
              try {
                await setLabel.mutateAsync({ accountId: props.account.id, labelId: label.id, assigned, invalidate: false });
                // Persist immediately, but let the row move/disappear only after closing.
                savedLabels.value = { ...savedLabels.value, [label.id]: assigned };
              } catch (error) {
                toast.add({
                  title: "Could not update labels",
                  description: error instanceof Error ? error.message : "Please try again.",
                  color: "error"
                });
              } finally {
                pendingLabels.value = pendingLabels.value.filter((item) => item !== pending);
                if (!open.value) await refreshAccounts();
              }
            }
          }))
          : [{ label: "No labels yet", disabled: true }]
  },
  {
    label: "Settings",
    icon: "i-lucide-settings",
    to: { name: "accounts-id-settings", params: { id: props.account.id } }
  },
  {
    label: "Delete account",
    icon: "i-lucide-trash-2",
    color: "error",
    onSelect: () => emit("delete")
  }
]);
</script>
