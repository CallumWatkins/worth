<template>
  <UInputMenu
    v-model:search-term="searchTerm"
    v-model:open="menuOpen"
    :model-value="
      /* Reka treats `undefined` as uncontrolled. Allow `null` to force clear on select.
      https://github.com/nuxt/ui/pull/6060 might fix the need for this workaround once released. */
      (selectedItem as SearchResultDto | undefined)
    "
    :items="searchQuery.data"
    label-key="name"
    leading-icon="i-lucide-search"
    :trailing-icon="false"
    open-on-focus
    ignore-filter
    :loading="showSearchingState && hasSearchTerm"
    placeholder="Search..."
    :ui="{
      content: 'min-w-fit',
      item: 'gap-2.5 items-center'
    }"
    :content="{ align: 'end' }"
    @update:model-value="onSelect"
  >
    <template #item-leading="{ item }">
      <UIcon
        :name="item.kind === 'account' ? 'i-lucide-wallet' : 'i-lucide-building-2'"
        class="size-5 text-muted"
      />
    </template>

    <template #item-label="{ item }">
      <div class="flex flex-col">
        <span>{{ item.name }}</span>
        <span
          v-if="item.kind === 'account' && item.institution_name"
          class="text-xs text-muted"
        >
          {{ item.institution_name }}
        </span>
      </div>
    </template>

    <template #item-trailing="{ item }">
      <UBadge
        v-if="item.kind === 'account' && item.account_type"
        variant="subtle"
        color="neutral"
        size="sm"
        :class="accountTypeBadgeClass(item.account_type)"
      >
        {{ accountTypeLabel(item.account_type) }}
      </UBadge>
    </template>

    <template #empty>
      <span v-if="showSearchingState">
        Searching...
      </span>
      <span v-else>
        No results
      </span>
    </template>
  </UInputMenu>
</template>

<script lang="ts" setup>
import type { SearchResultDto } from "~/bindings";

import { useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs, watch } from "vue";

import { accountTypeBadgeClass, accountTypeLabel } from "~/utils/account-type-meta";

const api = useApi();

const selectedItem = ref<SearchResultDto | null>(null);
const rawMenuOpen = ref(false);
const searchTerm = ref("");
const hasSearchTerm = computed(() => searchTerm.value.trim().length > 0);
const debouncedSearchTerm = refDebounced(searchTerm, 200);
const trimmedSearchTerm = computed(() => debouncedSearchTerm.value.trim());
const menuOpen = computed({
  get: () => rawMenuOpen.value && hasSearchTerm.value,
  set: (value: boolean) => {
    rawMenuOpen.value = value && hasSearchTerm.value;
  }
});

watch(hasSearchTerm, (hasValue) => {
  rawMenuOpen.value = hasValue;
});

const searchQuery = proxyRefs(useQuery({
  queryKey: ["search", "global", trimmedSearchTerm],
  enabled: computed(() => trimmedSearchTerm.value.length > 0),
  queryFn: () => api.search(trimmedSearchTerm.value)
}));
const showSearchingState = computed(() =>
  searchQuery.isFetching || (hasSearchTerm.value && searchTerm.value.trim() !== trimmedSearchTerm.value)
);

async function onSelect(item: SearchResultDto | undefined) {
  if (!item) return;

  selectedItem.value = null;
  searchTerm.value = "";
  menuOpen.value = false;

  if (item.kind === "account") {
    await navigateTo(`/accounts/${item.id}`);
  } else if (item.kind === "institution") {
    await navigateTo(`/institutions/${item.id}`);
  } else {
    assertNever(item);
  }
}
</script>
