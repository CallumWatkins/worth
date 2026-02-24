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
          v-model:show-empty="showEmpty"
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
        :title="accountsQuery.error!.message ?? 'Failed to load accounts'"
      />
      <AccountsTable
        :accounts="accountsQuery.data ?? []"
        :group-by="groupBy"
        :show-empty="showEmpty"
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
  showEmpty,
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
