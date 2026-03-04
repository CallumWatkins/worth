<template>
  <UContainer>
    <UPageHeader
      title="Institutions"
      description="View institutions and their account totals"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    >
      <template #links>
        <UButton
          label="Add New Institution"
          icon="i-lucide-plus"
          color="primary"
          variant="solid"
          @click="createDialogOpen = true"
        />
      </template>
    </UPageHeader>

    <UPageBody class="space-y-6">
      <UAlert
        v-if="institutionsQuery.isError"
        color="error"
        variant="subtle"
        :title="institutionsQuery.error!.message ?? 'Failed to load institutions'"
      />

      <UTable
        v-model:sorting="sorting"
        :data="institutionsData"
        :columns="columns"
        empty="No institutions available."
        :ui="{
          tr: 'data-[selectable=true]:cursor-pointer'
        }"
        class="flex-1"
        @select="onSelect"
      >
        <template #name-cell="{ row }">
          <span class="text-highlighted">
            {{ row.original.name }}
          </span>
        </template>

        <template #accounts-cell="{ row }">
          {{ row.original.account_count }}
        </template>

        <template #emptyAccounts-cell="{ row }">
          {{ row.original.empty_account_count }}
        </template>

        <template #accountTypes-cell="{ row }">
          <div
            v-if="row.original.account_types.length"
            class="flex items-center gap-1.5"
          >
            <UBadge
              v-for="accountType in row.original.account_types"
              :key="accountType"
              variant="subtle"
              color="neutral"
              :class="accountTypeBadgeClass(accountType)"
            >
              {{ accountTypeLabel(accountType) }}
            </UBadge>
          </div>
          <span v-else class="text-muted">-</span>
        </template>

        <template #balance-cell="{ row }">
          {{ formatGBP(row.original.total_balance_minor) }}
        </template>
      </UTable>
    </UPageBody>

    <InstitutionsCreateDialog v-model:open="createDialogOpen" />
  </UContainer>
</template>

<script lang="ts" setup>
import type { TableColumn, TableRow } from "@nuxt/ui";
import type { InstitutionSummaryDto } from "~/generated/bindings";
import { useQuery } from "@tanstack/vue-query";

type Institution = InstitutionSummaryDto;

const UButton = resolveComponent("UButton");

const api = useApi();
const createDialogOpen = ref(false);

const institutionsQuery = proxyRefs(useQuery({
  queryKey: queryKeys.institutions.list(),
  queryFn: api.institutionsList
}));

const institutionsData = computed<Institution[]>(() => institutionsQuery.data ?? []);

const sorting = ref([
  {
    id: "name",
    desc: false
  }
]);

const gbp = new Intl.NumberFormat("en-GB", {
  style: "currency",
  currency: "GBP"
});

function formatGBP(minor: number) {
  return gbp.format(minor / 100);
}

async function onSelect(_e: Event, row: TableRow<Institution>) {
  await navigateTo({ name: "institutions-id", params: { id: row.original.id } });
}

function sortableHeader(column: any, label: string) {
  const isSorted = column.getIsSorted();

  return h(UButton, {
    color: "neutral",
    variant: "ghost",
    label,
    trailing: !!isSorted,
    trailingIcon: isSorted
      ? (isSorted === "asc"
        ? "i-lucide-arrow-up-narrow-wide"
        : "i-lucide-arrow-down-wide-narrow")
      : undefined,
    class: "-mx-2.5",
    onClick: () => {
      const current = column.getIsSorted();
      if (current === "desc") {
        column.clearSorting();
        return;
      }
      column.toggleSorting(current === "asc");
    }
  });
}

function staticHeader(label: string) {
  return h("span", {
    class: "inline-flex items-center -mx-2.5 px-2.5 text-sm font-medium text-default"
  }, label);
}

const columns = computed<TableColumn<Institution>[]>(() => [
  {
    accessorKey: "name",
    header: ({ column }) => sortableHeader(column, "Name")
  },
  {
    id: "accounts",
    accessorFn: (row) => row.account_count,
    header: ({ column }) => sortableHeader(column, "Accounts"),
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  },
  {
    id: "emptyAccounts",
    accessorFn: (row) => row.empty_account_count,
    header: ({ column }) => sortableHeader(column, "Empty Accounts"),
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  },
  {
    id: "accountTypes",
    accessorFn: (row) => row.account_types.map(accountTypeLabel).join(", "),
    header: () => staticHeader("Account Types"),
    enableSorting: false,
    meta: {
      class: {
        th: "w-full",
        td: "w-full"
      }
    }
  },
  {
    id: "balance",
    accessorKey: "total_balance_minor",
    header: ({ column }) => sortableHeader(column, "Balance"),
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  }
]);
</script>
