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
        <UPageCard
          :ui="{
            body: 'w-full'
          }"
        >
          <template #body>
            <div class="flex flex-row items-center justify-between">
              <div>
                <div class="text-base text-pretty font-semibold text-highlighted">
                  Accounts
                </div>
                <div class="text-[15px] text-pretty text-muted mt-1">
                  Accounts at this institution
                </div>
              </div>
              <AccountsTableViewOptions
                v-model:group-by="groupBy"
                v-model:activity-period="activityPeriod"
                v-model:show-empty="showEmpty"
                :group-by-items="groupByItems"
                :activity-period-items="activityPeriodItems"
              />
            </div>
          </template>

          <AccountsTable
            :accounts="institutionQuery.data.accounts"
            :group-by="groupBy"
            :show-empty="showEmpty"
            :activity-period="activityPeriod"
            :hide-columns="hideColumns"
          />
        </UPageCard>
      </template>

      <AccountsCreateDialog
        v-model:open="createAccountOpen"
        :default-institution-id="institutionId"
      />
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem } from "@nuxt/ui";
import type { InstitutionDetailDto } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

type Institution = InstitutionDetailDto;

const route = useRoute("institutions-id");
const api = useApi();
const hideColumns = ref<AccountsHideColumn[]>(["institution"]);
const createAccountOpen = ref(false);
const {
  groupBy,
  groupByItems,
  showEmpty,
  activityPeriod,
  activityPeriodItems
} = useAccountsTableOptions({
  hideColumns
});

const institutionId = useRouteParamInt(route, "id");

const institutionQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.get(institutionId.value!)),
  enabled: computed(() => institutionId.value !== null),
  queryFn: () => api.institutionsGet(institutionId.value!)
}));

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
  const institution = institutionQuery.data as Institution | undefined;
  if (!institution) return "";
  return `${institution.accounts.length} account${institution.accounts.length === 1 ? "" : "s"}`;
});
</script>
