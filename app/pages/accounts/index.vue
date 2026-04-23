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
          v-model:group-by="groupBy"
          v-model:activity-period="activityPeriod"
          v-model:hide-empty="hideEmpty"
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
        :title="accountsQuery.error.message"
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
        :accounts="accountsQuery.data ?? []"
        :group-by="groupBy"
        :hide-empty="hideEmpty"
        :activity-period="activityPeriod"
        :hide-columns="hideColumns"
      />

      <AccountsCreateDialog v-model:open="createDialogOpen" />
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";

const api = useApi();
const hideColumns = ref<AccountsHideColumn[]>([]);
const createDialogOpen = ref(false);

const {
  groupBy,
  groupByItems,
  hideEmpty,
  activityPeriod,
  activityPeriodItems
} = useAccountsTableOptions({
  hideColumns
});

const accountsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.accounts.list(),
  queryFn: api.accountsList
}));
</script>
