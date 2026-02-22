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
      No results
    </template>
  </UInputMenu>
</template>

<script lang="ts" setup>
import type { SearchResultDto } from "~/bindings";

import { keepPreviousData, useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs, watch } from "vue";

import { accountTypeBadgeClass, accountTypeLabel } from "~/utils/account-type-meta";

const api = useApi();

const selectedItem = ref<SearchResultDto | null>(null);
const rawMenuOpen = ref(false);
const searchTerm = ref("");
const trimmedSearchTerm = computed(() => searchTerm.value.trim());
const hasSearchTerm = computed(() => trimmedSearchTerm.value.length > 0);
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
  enabled: hasSearchTerm,
  queryFn: () => api.search(trimmedSearchTerm.value),
  // Prevent "no results" flash while typing by keeping previous data until the new query completes
  placeholderData: keepPreviousData
}));

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
