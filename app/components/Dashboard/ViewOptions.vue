<template>
  <UPopover
    arrow
    :content="{ align: 'end', side: 'bottom', sideOffset: 8 }"
    :ui="{ content: 'p-4 w-80' }"
  >
    <UButton
      label="View options"
      icon="i-lucide-sliders-horizontal"
      color="neutral"
      variant="subtle"
    />

    <template #content="{ close }">
      <div class="flex items-center justify-between gap-4 mb-4">
        <div class="font-semibold text-highlighted">
          View options
        </div>
        <UButton
          icon="i-lucide-x"
          color="neutral"
          variant="ghost"
          aria-label="Close view options"
          @click="close"
        />
      </div>

      <UFormField label="Accounts" name="accounts" :error="errorMessage">
        <USelectMenu
          :model-value="includedIds"
          :items="accountItems"
          value-key="value"
          multiple
          :filter-fields="['label', 'institution', 'accountType']"
          :reset-search-term-on-select="false"
          :search-input="searchInputProps"
          placeholder="None"
          color="neutral"
          variant="subtle"
          class="w-full"
          :loading="accountsQuery.isPending"
          :disabled="!accountsQuery.isSuccess"
          :content="{ align: 'end' }"
          :ui="{
            content: 'min-w-96 max-w-[calc(100vw-2rem)] max-h-[min(28rem,var(--reka-combobox-content-available-height,28rem))]',
            item: 'gap-2.5 items-center',
            itemLabel: 'w-full',
            itemTrailingIcon: 'hidden'
          }"
          @update:model-value="onSelectionChange"
        >
          <template #default>
            <span class="truncate">Showing {{ includedIds.length }} of {{ accounts.length }}</span>
          </template>
          <template #trailing="{ ui }">
            <UButton
              v-if="includedIds.length < accounts.length"
              as="span"
              role="button"
              tabindex="0"
              color="neutral"
              variant="link"
              aria-label="Show all accounts"
              :class="ui.trailingClear()"
              @click.stop="showAllAccounts"
              @keydown.enter.space.stop.prevent="showAllAccounts"
            />
            <UIcon v-else name="i-lucide-chevron-down" :class="ui.trailingIcon()" />
          </template>
          <template #item-label="{ item }">
            <div v-if="item.type === 'item'" class="flex items-center justify-between gap-4 min-w-0">
              <div class="flex flex-col min-w-0">
                <span class="truncate">{{ item.label }}</span>
                <span class="text-xs text-muted truncate">{{ item.institution }}</span>
              </div>
              <span class="text-sm text-toned tabular-nums whitespace-nowrap">
                {{ item.balance }}
              </span>
            </div>
          </template>
          <template #item-trailing="{ item }">
            <UIcon
              v-if="item.type === 'item'"
              :name="includedIds.includes(item.value) ? 'i-lucide-eye' : 'i-lucide-eye-off'"
              class="size-4 shrink-0 text-muted"
              aria-hidden="true"
            />
          </template>
        </USelectMenu>
      </UFormField>
    </template>
  </UPopover>
</template>

<script lang="ts" setup>
import type { AccountTypeName } from "~/generated/bindings";

const { formatCurrencyMinor } = useLocaleFormatters();
const {
  accountsQuery,
  accounts,
  includedIds,
  errorMessage,
  onSelectionChange,
  showAllAccounts
} = useDashboardAccountSelection();

// Keep a direct input reference: Nuxt UI's internal ref currently resolves to
// UInput's wrapper, so its automatic focus restoration cannot focus the input.
const searchInput = shallowRef<HTMLInputElement>();
const searchInputProps = {
  placeholder: "Search accounts...",
  onFocus: (event: FocusEvent) => {
    searchInput.value = event.target as HTMLInputElement;
  }
};

const accountItems = computed(() => (Object.keys(ACCOUNT_TYPE_META) as AccountTypeName[]).map((type) => {
  const meta = ACCOUNT_TYPE_META[type];
  const accountsOfType = accounts.value
    .filter((account) => account.account_type.name === type)
    .toSorted((a, b) => a.name.localeCompare(b.name) || a.institution.name.localeCompare(b.institution.name));

  if (!accountsOfType.length) return [];

  return [
    {
      type: "label" as const,
      label: meta.label,
      value: undefined,
      ui: { label: `w-fit rounded-sm mx-1 my-1 px-2 py-1 ${meta.badgeClass}` }
    },
    ...accountsOfType.map((account) => ({
      type: "item" as const,
      label: account.name,
      value: account.id,
      institution: account.institution.name,
      accountType: meta.label,
      balance: formatCurrencyMinor(account.latest_balance_minor, account.currency_code),
      onSelect: () => searchInput.value?.focus({ preventScroll: true })
    }))
  ];
}).filter((group) => group.length));
</script>
