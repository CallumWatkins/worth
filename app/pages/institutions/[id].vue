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
          description="Placeholder for accounts table"
        />
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem } from "@nuxt/ui";
import type { InstitutionDto } from "~/bindings";

import { useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs } from "vue";

type Institution = InstitutionDto;

const route = useRoute();
const api = useApi();

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
  return "Institution overview";
});
</script>
