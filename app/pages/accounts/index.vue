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
          to="/accounts/new"
          color="primary"
          variant="solid"
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
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { AccountsHideColumn } from "~/composables/useAccountsTableOptions";

import { useQuery } from "@tanstack/vue-query";
import { proxyRefs } from "vue";

const api = useApi();
const hideColumns = ref<AccountsHideColumn[]>([]);

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
  queryKey: ["accounts", "list"],
  queryFn: api.accountsList
}));
</script>
