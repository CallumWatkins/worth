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
          v-model:hide-empty="options.hideEmpty"
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

      <AccountsTable
        v-else
        v-model:sorting="options.sorting"
        v-model:expanded="options.expanded"
        :accounts="accountsQuery.data ?? []"
        :group-by="options.groupBy"
        :hide-empty="options.hideEmpty"
        :activity-period="options.activityPeriod"
        :hide-columns="hideColumns"
        analytics-category="accounts"
        @clear-filters="options.hideEmpty = false"
      />

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
