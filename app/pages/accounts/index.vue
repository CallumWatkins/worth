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
        <UPopover
          arrow
          :content="{ align: 'end', side: 'bottom', sideOffset: 8 }"
          :ui="{ content: 'p-4 w-80' }"
        >
          <UButton
            label="View options"
            icon="i-lucide-sliders-horizontal"
            color="neutral"
            variant="subtle"
          />

          <template #content="{ close }">
            <div class="flex items-center justify-between gap-4 mb-4">
              <div class="font-semibold text-highlighted">
                View options
              </div>
              <UButton
                icon="i-lucide-x"
                color="neutral"
                variant="ghost"
                @click="close"
              />
            </div>

            <div class="space-y-4">
              <UFormField name="groupBy" label="Group accounts by">
                <USelect
                  v-model="groupBy"
                  :items="groupByItems"
                  class="w-full"
                  color="neutral"
                  variant="subtle"
                />
              </UFormField>

              <UFormField name="activityPeriod" label="Activity period">
                <USelect
                  v-model="activityPeriod"
                  :items="activityPeriodItems"
                  class="w-full"
                  color="neutral"
                  variant="subtle"
                />
              </UFormField>

              <UFormField
                name="showEmpty"
                label="Show empty accounts"
                orientation="horizontal"
                class="items-center"
              >
                <UCheckbox v-model="showEmpty" color="neutral" />
              </UFormField>
            </div>
          </template>
        </UPopover>

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
      <UTable
        v-model:sorting="sorting"
        v-model:column-visibility="columnVisibility"
        :data="accountsData"
        :columns="columns"
        :grouping="grouping"
        :grouping-options="groupingOptions"
        empty="No accounts match these filters."
        :ui="{
          td: 'empty:p-0',
          tr: 'data-[selectable=true]:cursor-pointer'
        }"
        class="flex-1"
        @select="onSelect"
      >
        <template #name-cell="{ row }">
          <div class="flex items-center gap-2">
            <span
              class="inline-block"
              :style="{ width: `calc(${row.depth} * 1rem)` }"
            />

            <UButton
              v-if="grouping.length"
              variant="outline"
              color="neutral"
              size="xs"
              class="shrink-0"
              :icon="row.getIsExpanded() ? 'i-lucide-minus' : 'i-lucide-plus'"
              :class="!row.getCanExpand() ? 'invisible' : ''"
              :ui="{
                base: 'p-0 rounded-sm',
                leadingIcon: 'size-4'
              }"
              @click.stop="row.toggleExpanded()"
            />

            <div v-if="row.getIsGrouped()" class="flex items-center gap-2">
              <span class="font-semibold text-highlighted">
                {{ getGroupLabel(row) }}
              </span>
              <UBadge variant="subtle" color="neutral">
                {{ row.subRows?.length || 0 }}
              </UBadge>
            </div>

            <div v-else class="flex items-center gap-2 min-w-0">
              <span class="font-medium text-highlighted truncate">
                {{ row.original.name }}
              </span>
              <UBadge
                v-if="row.original.latest_balance_minor === 0"
                variant="subtle"
                color="neutral"
              >
                Empty
              </UBadge>
            </div>
          </div>
        </template>

        <template #institution-cell="{ row }">
          <span v-if="row.getIsGrouped()" />
          <span v-else>
            {{ row.original.institution.name }}
          </span>
        </template>

        <template #type-cell="{ row }">
          <span v-if="row.getIsGrouped()" />
          <UBadge
            v-else
            variant="subtle"
            color="neutral"
            :class="accountTypeBadgeClass(row.original.account_type.name)"
          >
            {{ accountTypeLabel(row.original.account_type.name) }}
          </UBadge>
        </template>

        <template #firstChange-cell="{ row }">
          <span v-if="row.getIsGrouped()">
            {{ formatShortDate(getGroupedFirstChange(row)) }}
          </span>
          <span v-else>
            {{ formatShortDate(row.original.first_snapshot_date) }}
          </span>
        </template>

        <template #lastChange-cell="{ row }">
          <span v-if="row.getIsGrouped()">
            {{ formatShortDate(getGroupedLastChange(row)) }}
          </span>
          <span v-else>
            {{ formatShortDate(row.original.latest_snapshot_date) }}
          </span>
        </template>

        <template #activity-cell="{ row }">
          <div class="flex justify-end">
            <svg
              viewBox="0 0 120 28"
              class="h-7 w-28"
              aria-hidden="true"
            >
              <path
                :d="sparklinePath(getRowActivityValues(row))"
                :stroke="getRowActivityColor(row)"
                stroke-width="2"
                fill="none"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
        </template>

        <template #balance-cell="{ row }">
          <div v-if="row.getIsGrouped()" class="text-right">
            <div class="font-semibold text-highlighted">
              {{ formatGBP(getGroupedBalance(row)) }}
            </div>
            <div class="text-xs text-muted">
              Group total
            </div>
          </div>
          <span v-else class="font-medium">
            {{ formatGBP(row.original.latest_balance_minor) }}
          </span>
        </template>
      </UTable>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { SelectItem, TableColumn, TableRow } from "@nuxt/ui";
import type { GroupingOptions } from "@tanstack/vue-table";
import type { AccountDto, AccountTypeName, ActivityPeriod } from "~/bindings";

import { useQuery } from "@tanstack/vue-query";
import { getGroupedRowModel } from "@tanstack/vue-table";
import { h, proxyRefs, resolveComponent } from "vue";

import { accountTypeBadgeClass, accountTypeLabel, accountTypeLineColor } from "~/utils/account-type-meta";

type Account = AccountDto;

const UButton = resolveComponent("UButton");

const groupByItems = ref<SelectItem[]>([
  { label: "None", value: "none" },
  { label: "Institution", value: "institution" },
  { label: "Type", value: "type" }
]);

const groupBy = ref<"none" | "institution" | "type">("none");
const showEmpty = ref(false);

const activityPeriodItems = ref<SelectItem[]>([
  { label: "1W", value: "1W" },
  { label: "1M", value: "1M" },
  { label: "3M", value: "3M" },
  { label: "6M", value: "6M" }
]);

const activityPeriod = ref<ActivityPeriod>("1M");

const api = useApi();

const accountsQuery = proxyRefs(useQuery({
  queryKey: ["accounts", "list"],
  queryFn: api.accountsList
}));

const rawAccounts = computed<Account[]>(() => accountsQuery.data ?? []);

const accountsData = computed(() => {
  if (showEmpty.value) {
    return rawAccounts.value;
  }

  return rawAccounts.value.filter((a) => a.latest_balance_minor !== 0);
});

const grouping = computed(() => {
  if (groupBy.value === "institution") {
    return ["institution_group"];
  }
  if (groupBy.value === "type") {
    return ["type_group"];
  }
  return [];
});

const groupingOptions = ref<GroupingOptions>({
  groupedColumnMode: "remove",
  getGroupedRowModel: getGroupedRowModel()
});

const columnVisibility = ref<Record<string, boolean>>({
  institution_group: false,
  type_group: false
});

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

function getGroupedFirstChange(row: TableRow<Account>) {
  return String(row.getValue("firstChange") ?? "");
}

function activityValues(account: Account, period: ActivityPeriod): Array<number | null> {
  return account.activity_by_period?.[period]?.values ?? [];
}

function deltaFromValues(values: Array<number | null>) {
  const first = values.find((v) => typeof v === "number");
  const last = values.at(-1);

  if (typeof first !== "number" || typeof last !== "number") {
    return 0;
  }

  return last - first;
}

function leafAccountsFromRow(row: TableRow<Account>) {
  const out: Account[] = [];

  const walk = (r: TableRow<Account>) => {
    if (r.getIsGrouped()) {
      (r.subRows || []).forEach(walk);
      return;
    }

    out.push(r.original);
  };

  walk(row);
  return out;
}

function aggregateSeries(seriesList: Array<Array<number | null>>) {
  const length = seriesList[0]?.length ?? 0;
  const out = Array.from({ length }, () => null as number | null);

  for (let i = 0; i < length; i++) {
    let sum = 0;
    let hasValue = false;

    for (const series of seriesList) {
      const v = series[i];
      if (typeof v === "number") {
        sum += v;
        hasValue = true;
      }
    }

    out[i] = hasValue ? sum : null;
  }

  return out;
}

function getRowActivityValues(row: TableRow<Account>) {
  if (!row.getIsGrouped()) {
    return activityValues(row.original, activityPeriod.value);
  }

  const accounts = leafAccountsFromRow(row);
  const seriesList = accounts
    .map((a) => activityValues(a, activityPeriod.value))
    .filter((v) => v.length > 0) as Array<Array<number | null>>;

  return aggregateSeries(seriesList);
}

function getRowActivityColor(row: TableRow<Account>) {
  if (!row.getIsGrouped()) {
    return accountTypeLineColor(row.original.account_type.name);
  }

  const groupingId = row.groupingColumnId as string | undefined;
  if (groupingId === "type_group") {
    return accountTypeLineColor(row.getValue("type_group") as AccountTypeName);
  }

  // Institution (or unknown) groups: neutral line
  return "#94A3B8";
}

function sparklinePath(values: Array<number | null>) {
  const width = 120;
  const height = 28;
  const paddingY = 2;

  if (values.length < 2) {
    return "";
  }

  let min = Infinity;
  let max = -Infinity;
  let hasAny = false;

  for (const v of values) {
    if (typeof v !== "number") {
      continue;
    }
    hasAny = true;
    min = Math.min(min, v);
    max = Math.max(max, v);
  }

  if (!hasAny) {
    return "";
  }

  const usableHeight = Math.max(1, height - paddingY * 2);
  const range = max - min;
  const denom = values.length - 1;

  let d = "";
  let started = false;

  for (let i = 0; i < values.length; i++) {
    const v = values[i];
    if (typeof v !== "number") {
      started = false;
      continue;
    }

    const x = (i / denom) * width;
    const y = range === 0
      ? paddingY + usableHeight / 2
      : paddingY + ((max - v) / range) * usableHeight;

    const xf = x.toFixed(2);
    const yf = y.toFixed(2);

    if (!started) {
      d += `M ${xf} ${yf}`;
      started = true;
    } else {
      d += ` L ${xf} ${yf}`;
    }
  }

  return d;
}

function parseIsoDate(iso: string) {
  // Avoid timezone shifts for date-only strings.
  return new Date(`${iso}T00:00:00`);
}

function formatShortDate(iso: string) {
  return parseIsoDate(iso).toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "short",
    year: "numeric"
  });
}

function getGroupLabel(row: TableRow<Account>) {
  const id = row.groupingColumnId as string | undefined;
  if (id === "institution_group") {
    return String(row.getValue("institution_group") ?? "");
  }
  if (id === "type_group") {
    return accountTypeLabel(row.getValue("type_group") as AccountTypeName);
  }
  return String(id ? row.getValue(id) : "");
}

function getGroupedBalance(row: TableRow<Account>) {
  return Number(row.getValue("balance") ?? 0);
}

function getGroupedLastChange(row: TableRow<Account>) {
  return String(row.getValue("lastChange") ?? "");
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

const columns = computed<TableColumn<Account>[]>(() => [
  {
    id: "institution_group",
    accessorFn: (row) => row.institution.name,
    enableSorting: false
  },
  {
    id: "type_group",
    accessorFn: (row) => row.account_type.name,
    enableSorting: false
  },
  {
    accessorKey: "name",
    header: ({ column }) => sortableHeader(column, "Name")
  },
  {
    id: "institution",
    accessorFn: (row) => row.institution.name,
    header: ({ column }) => sortableHeader(column, "Institution")
  },
  {
    id: "type",
    accessorFn: (row) => row.account_type.name,
    header: ({ column }) => sortableHeader(column, "Type")
  },
  {
    id: "firstChange",
    header: ({ column }) => sortableHeader(column, "First change"),
    accessorKey: "first_snapshot_date",
    aggregationFn: (_columnId, leafRows: any[]) => {
      const dates = leafRows
        .map((r) => String(r?.original?.first_snapshot_date ?? ""))
        .filter(Boolean);

      return dates.length
        ? dates.reduce((min: string, d: string) => (d < min ? d : min), dates[0]!)
        : "";
    }
  },
  {
    id: "lastChange",
    accessorKey: "latest_snapshot_date",
    header: ({ column }) => sortableHeader(column, "Last change"),
    aggregationFn: "max"
  },
  {
    id: "activity",
    header: ({ column }) => sortableHeader(column, `Activity (${activityPeriod.value})`),
    accessorFn: (row) => row.activity_by_period?.[activityPeriod.value]?.delta_minor ?? 0,
    aggregationFn: (_columnId, leafRows: any[]) => {
      const period = activityPeriod.value;
      const seriesList = leafRows
        .map((r) => r?.original?.activity_by_period?.[period]?.values)
        .filter(Boolean) as Array<Array<number | null>>;

      return deltaFromValues(aggregateSeries(seriesList));
    },
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  },
  {
    id: "balance",
    accessorKey: "latest_balance_minor",
    header: ({ column }) => sortableHeader(column, "Balance"),
    aggregationFn: "sum",
    meta: {
      class: {
        th: "text-right",
        td: "text-right font-medium"
      }
    }
  }
]);

function onSelect(_e: Event, row: TableRow<Account>) {
  if (row.getIsGrouped()) {
    row.toggleExpanded();
    return;
  }

  void navigateTo(`/accounts/${row.original.id}`);
}
</script>
