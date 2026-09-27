<template>
  <UContainer>
    <div v-if="institutionQuery.isSuccess" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="institutionQuery.isSuccess"
      :title="institutionQuery.data.name"
      :description="headerDescription"
      :ui="{
        root: 'pb-0 border-none',
        title: 'text-balance',
        links: 'flex-nowrap',
        description: 'mt-1'
      }"
    >
      <template #links>
        <UButton
          label="Settings"
          icon="i-lucide-settings"
          color="neutral"
          variant="subtle"
          :to="{ name: 'institutions-id-settings', params: { id: institutionQuery.data.id } }"
        />
        <UButton
          label="Add account"
          icon="i-lucide-plus"
          color="primary"
          variant="solid"
          @click="createAccountOpen = true"
        />
      </template>
    </UPageHeader>

    <UPageBody class="space-y-8">
      <template v-if="institutionQuery.isSuccess">
        <EmptyPageState
          v-if="institutionQuery.data.accounts.length === 0"
          icon="i-lucide-wallet"
          title="No accounts in this institution yet"
          description="Create an account to start tracking balance snapshots over time."
          action-label="Create account"
          action-icon="i-lucide-plus"
          @action="createAccountOpen = true"
        />

        <UPageCard
          v-else
          :ui="{
            body: 'w-full',
            container: 'grid'
          }"
        >
          <template #body>
            <div class="flex flex-row items-center justify-between">
              <div>
                <div class="text-base font-semibold text-highlighted">
                  Accounts
                </div>
                <div class="text-[15px] text-muted mt-1">
                  Accounts at this institution
                </div>
              </div>
              <AccountsTableViewOptions
                v-model:group-by="options.groupBy"
                v-model:activity-period="options.activityPeriod"
                :group-by-items="groupByItems"
                :activity-period-items="activityPeriodItems"
              />
            </div>
          </template>

          <div class="flex flex-wrap items-center gap-3">
            <FiltersMenu
              v-model="options.filters.types"
              label="Type"
              :items="typeItems"
            />
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
            :total-count="institutionQuery.data.accounts.length"
            :group-by="options.groupBy"
            :activity-period="options.activityPeriod"
            :hide-columns="hideColumns"
            analytics-category="institution"
            @show-all="resetFilters"
          />
        </UPageCard>
      </template>

      <AccountsCreateDialog
        v-model:open="createAccountOpen"
        :default-institution-id="institutionId"
        analytics-category="institution"
      />
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem } from "@nuxt/ui";
import { useQuery } from "@tanstack/vue-query";

const route = useRoute("institutions-id");
const institutionId = useRouteParamInt(route, "id");
const api = useApi();
const hideColumns = ref<AccountsHideColumn[]>(["institution"]);
const createAccountOpen = ref(false);
const {
  options,
  groupByItems,
  activityPeriodItems
} = useAccountsTableOptions({
  scope: () => `institution:${institutionId.value}`,
  hideColumns
});

const institutionQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.get(institutionId.value!)),
  enabled: computed(() => institutionId.value !== null),
  queryFn: async () => api.institutionsGet(institutionId.value!)
}));

const { typeItems, statusItems, balanceItems, hasFilters, filteredAccounts, resetFilters } = useAccountFilters(
  () => institutionQuery.data?.accounts ?? [],
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
      if (institutionQuery.isSuccess) {
        createAccountOpen.value = true;
      }
    }
  }
]);

useResourcePageError({
  resourceName: "Institution",
  resourceId: institutionId,
  resourceIsError: computed(() => institutionQuery.isError),
  resourceError: computed(() => institutionQuery.error),
  fallbackErrorMessage: "Failed to load institution"
});

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const institution = institutionQuery.data;
  return [
    { label: "Institutions", to: { name: "institutions" }, icon: "i-lucide-building-2" },
    { label: institution?.name ?? "" }
  ];
});

const headerDescription = computed(() => {
  const institution = institutionQuery.data;
  if (!institution) return "";
  return `${institution.accounts.length} account${institution.accounts.length === 1 ? "" : "s"}`;
});
</script>
