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
    />

    <UPageBody class="space-y-8">
      <UAlert
        v-if="invalidId"
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
          <template #body>
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
          </template>
        </UPageCard>
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem } from "@nuxt/ui";
import type { InstitutionDetailDto } from "~/bindings";
import type { AccountsHideColumn } from "~/composables/useAccountsTableOptions";

import { useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs } from "vue";

type Institution = InstitutionDetailDto;

const route = useRoute();
const api = useApi();
const hideColumns = ref<AccountsHideColumn[]>(["institution"]);
const {
  groupBy,
  groupByItems,
  showEmpty,
  activityPeriod,
  activityPeriodItems
} = useAccountsTableOptions({
  hideColumns
});

const rawId = computed(() => {
  const p = (route.params as any)?.id;
  if (Array.isArray(p)) return p[0];
  return p;
});

const institutionId = computed<number | null>(() => {
  const s = String(rawId.value ?? "");
  const n = Number.parseInt(s, 10);
  if (!Number.isFinite(n)) return null;
  return n;
});

const invalidId = computed(() => rawId.value != null && institutionId.value == null);

const institutionQuery = proxyRefs(useQuery({
  queryKey: ["institutions", "get", institutionId],
  enabled: computed(() => typeof institutionId.value === "number"),
  queryFn: () => api.institutionsGet(institutionId.value as number)
}));

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const institution = institutionQuery.data;
  return [
    { label: "Institutions", to: "/institutions", icon: "i-lucide-building-2" },
    { label: institution?.name ?? "" }
  ];
});

const headerDescription = computed(() => {
  const institution = institutionQuery.data as Institution | undefined;
  if (!institution) return "";
  return `${institution.accounts.length} account${institution.accounts.length === 1 ? "" : "s"}`;
});
</script>
