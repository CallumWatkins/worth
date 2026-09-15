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
          <AccountsFilterMenu
            v-model="options.filters.institutionIds"
            label="Institution"
            :items="institutionItems"
          />
          <AccountsFilterMenu
            v-model="options.filters.types"
            label="Type"
            :items="typeItems"
          />
          <AccountsFilterMenu
            v-model="options.filters.statuses"
            label="Status"
            :items="statusItems"
          />
          <AccountsBalanceFilter
            v-model="options.filters.balances"
            v-model:range="options.filters.balanceRange"
            :items="balanceItems"
          />
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

const { institutionItems, typeItems, statusItems, balanceItems, hasFilters, filteredAccounts, resetFilters } = useAccountFilters(
  () => accountsQuery.data ?? [],
  computed({
    get: () => options.value.filters,
    set: (filters) => {
      options.value.filters = filters;
    }
  })
);

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
