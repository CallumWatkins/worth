<template>
  <UContainer>
    <div v-if="institutionQuery.data" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="institutionQuery.data"
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
      <UAlert
        v-if="institutionId == null"
        color="error"
        variant="subtle"
        title="Invalid institution id"
      />

      <UAlert
        v-else-if="institutionQuery.isError"
        color="error"
        variant="subtle"
        :title="institutionQuery.error!.message ?? 'Failed to load institution'"
      />

      <template v-else-if="institutionQuery.data">
        <UPageCard
          title="Accounts"
          description="Accounts at this institution"
          :ui="{ body: 'min-w-full' }"
        >
          <div class="flex items-center justify-end gap-4 mb-4">
            <AccountsTableViewOptions
              v-model:group-by="groupBy"
              v-model:activity-period="activityPeriod"
              v-model:show-empty="showEmpty"
              :group-by-items="groupByItems"
              :activity-period-items="activityPeriodItems"
            />
          </div>

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

const institutionId = computed<number | null>(() => {
  const n = Number.parseInt(route.params.id);
  if (!Number.isFinite(n)) return null;
  return n;
});

const institutionQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.institutions.get(institutionId.value!)),
  enabled: computed(() => institutionId.value !== null),
  queryFn: () => api.institutionsGet(institutionId.value!)
}));

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
