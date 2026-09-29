<template>
  <UContainer>
    <UPageHeader
      title="Accounts"
      description="Manage your accounts and their balances"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    >
      <template #links>
        <AccountsTableViewOptions
          v-model:group-by="options.groupBy"
          v-model:activity-period="options.activityPeriod"
          :group-by-items="groupByItems"
          :activity-period-items="activityPeriodItems"
        />

        <UButton
          label="Add New Account"
          icon="i-lucide-plus"
          color="primary"
          variant="solid"
          @click="createDialogOpen = true"
        />
      </template>
    </UPageHeader>
    <UPageBody class="space-y-6">
      <UAlert
        v-if="accountsQuery.isError"
        color="error"
        variant="subtle"
        orientation="horizontal"
        :title="accountsQuery.error.message"
        :actions="hasErrorDetailsSurvey ? [getErrorDetailsSurveyAction()] : []"
      />

      <EmptyPageState
        v-if="accountsQuery.isSuccess && accountsQuery.data.length === 0"
        icon="i-lucide-wallet"
        title="No accounts yet"
        description="Create an account to start tracking balance snapshots over time."
        action-label="Create account"
        action-icon="i-lucide-plus"
        @action="createDialogOpen = true"
      />

      <div v-else class="space-y-6">
        <div class="flex flex-wrap items-center gap-3">
          <FiltersMenu
            v-model="options.filters.institutionIds"
            label="Institution"
            :items="institutionItems"
          />
          <FiltersMenu
            v-model="options.filters.types"
            label="Type"
            :items="typeItems"
          />
          <FiltersMenu v-model="options.filters.labels" label="Label" :items="labelItems" />
          <FiltersMenu
            v-model="options.filters.statuses"
            label="Status"
            :items="statusItems"
          />
          <FiltersMenu
            v-model="options.filters.balances"
            label="Balance"
            :items="balanceItems"
            :active="options.filters.balanceRange.minimum !== null || options.filters.balanceRange.maximum !== null"
            @clear="options.filters.balanceRange = createBalanceRange()"
          >
            <template #item-label="{ item }">
              {{ item.label }} <span class="text-muted">({{ item.value === 'active' ? '≠ 0' : '= 0' }})</span>
            </template>
            <template #content-bottom>
              <FiltersBalanceRangeInputs v-model="options.filters.balanceRange" class="border-t border-default" />
            </template>
          </FiltersMenu>
          <UButton
            v-if="hasFilters"
            label="Reset filters"
            color="neutral"
            variant="link"
            @click="resetFilters"
          />
        </div>

        <AccountsTable
          v-model:sorting="options.sorting"
          v-model:expanded="options.expanded"
          :accounts="filteredAccounts"
          :total-count="accountsQuery.data?.length ?? 0"
          :group-by="options.groupBy"
          :activity-period="options.activityPeriod"
          :hide-columns="hideColumns"
          analytics-category="accounts"
          @show-all="resetFilters"
        />
      </div>

      <AccountsCreateDialog
        v-model:open="createDialogOpen"
        analytics-category="accounts"
      />
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";

const api = useApi();
const { hasErrorDetailsSurvey, getErrorDetailsSurveyAction } = useErrorDetailsSurvey();
const hideColumns = ref<AccountsHideColumn[]>([]);
const createDialogOpen = ref(false);

const {
  options,
  groupByItems,
  activityPeriodItems
} = useAccountsTableOptions({
  scope: "accounts",
  hideColumns
});

const accountsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.accounts.list(),
  queryFn: api.accountsList
}));

const { institutionItems, labelItems, typeItems, statusItems, balanceItems, hasFilters, filteredAccounts, resetFilters } = useAccountFilters(
  () => accountsQuery.data ?? [],
  computed({
    get: () => options.value.filters,
    set: (filters) => {
      options.value.filters = filters;
    }
  })
);

const route = useRoute("accounts");
const router = useRouter();
// Label links are one-time filter requests. Consume the query so another click
// on the same label can reset filters again; ordinary navigation keeps view state.
watch(() => route.query.label, (value) => {
  if (typeof value !== "string" || !/^[1-9]\d*$/.test(value)) return;
  const labelId = Number(value);
  if (!Number.isSafeInteger(labelId)) return;
  resetFilters();
  options.value.filters.labels = [labelId];
  void router.replace({ name: "accounts", query: {} });
}, { immediate: true });

useContextualKeyboardShortcuts([
  {
    label: "Add new account",
    combos: [["meta", "N"]],
    handler: () => {
      createDialogOpen.value = true;
    }
  }
]);
</script>
