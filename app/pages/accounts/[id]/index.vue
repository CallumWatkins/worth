<template>
  <UContainer>
    <div v-if="accountQuery.isSuccess" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="accountQuery.isSuccess"
      :title="accountQuery.data.name"
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
          :to="{ name: 'accounts-id-settings', params: { id: accountQuery.data.id } }"
        />
      </template>
    </UPageHeader>

    <UPageBody class="space-y-8">
      <template v-if="accountQuery.isSuccess">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <UPageCard
            title="Current Balance"
            variant="subtle"
            :ui="{ title: 'text-muted text-sm whitespace-nowrap' }"
          >
            <template #description>
              <div class="text-xl font-bold text-default whitespace-nowrap">
                {{ formatCurrencyMinor(accountQuery.data.latest_balance_minor, accountQuery.data.currency_code) }}
              </div>
              <div v-if="accountQuery.data.latest_snapshot_date != null" class="text-xs text-muted mt-1">
                As of {{ formatShortDate(accountQuery.data.latest_snapshot_date) }}
              </div>
            </template>
          </UPageCard>

          <UPageCard
            title="Institution"
            :description="accountQuery.data.institution.name"
            :to="{ name: 'institutions-id', params: { id: accountQuery.data.institution.id } }"
            variant="subtle"
            :ui="{
              title: 'text-muted text-sm whitespace-nowrap',
              description: 'text-xl font-bold text-default whitespace-nowrap'
            }"
          />

          <UPageCard
            title="Account Type"
            variant="subtle"
            :ui="{
              title: 'text-muted text-sm whitespace-nowrap',
              description: 'mt-2'
            }"
          >
            <template #description>
              <div class="flex items-center gap-2">
                <UBadge
                  variant="subtle"
                  color="neutral"
                  size="xl"
                  :class="ACCOUNT_TYPE_META[accountQuery.data.account_type.name].badgeClass"
                >
                  {{ ACCOUNT_TYPE_META[accountQuery.data.account_type.name].label }}
                </UBadge>
              </div>
            </template>
          </UPageCard>

          <UPageCard
            title="Monthly Change"
            variant="subtle"
            :ui="{
              title: 'text-muted text-sm whitespace-nowrap'
            }"
          >
            <template #description>
              <div
                v-if="monthlyChangeMinor != null"
                class="text-xl font-bold whitespace-nowrap"
                :class="monthlyChangeMinor >= 0 ? 'text-success' : 'text-error'"
              >
                {{ formatCurrencyMinor(monthlyChangeMinor, accountQuery.data.currency_code, { signDisplay: "always" }) }}
              </div>
              <div v-else class="text-xl font-bold text-default whitespace-nowrap">
                No change
              </div>
              <div class="text-xs text-muted mt-1">
                Over the last 30 days
              </div>
            </template>
          </UPageCard>
        </div>

        <EmptyPageState
          v-if="snapshotsQuery.isSuccess && snapshotsQuery.data.length === 0"
          icon="i-lucide-chart-line"
          title="No snapshots yet"
          description="Add balance snapshots to start building this account's history."
          action-label="Add snapshots"
          action-icon="i-lucide-plus"
          @action="addSnapshotsOpen = true"
        />

        <template v-else>
          <UPageCard
            :ui="{
              body: 'w-full',
              spotlight: 'bg-default/95'
            }"
          >
            <template #body>
              <div class="flex flex-row items-center justify-between">
                <div>
                  <div class="text-base text-pretty font-semibold text-highlighted">
                    Balance Over Time
                  </div>
                  <div class="text-[15px] text-pretty text-muted mt-1">
                    Growth trajectory over
                    <template v-if="balanceOverTimePeriod === '1M'">
                      the last month
                    </template>
                    <template v-else-if="balanceOverTimePeriod === '6M'">
                      the last 6 months
                    </template>
                    <template v-else-if="balanceOverTimePeriod === '1Y'">
                      the last year
                    </template>
                    <template v-else-if="balanceOverTimePeriod === 'MAX'">
                      all time
                    </template>
                  </div>
                </div>
                <div class="mt-4 sm:mt-0 sm:ml-6 shrink-0">
                  <UTabs
                    v-model="balanceOverTimePeriod"
                    :items="balanceOverTimePeriodItems"
                    :content="false"
                    color="neutral"
                    size="sm"
                    :ui="{
                      indicator: 'bg-neutral-700',
                      label: 'text-neutral-300'
                    }"
                  />
                </div>
              </div>
            </template>

            <UAlert
              v-if="balanceOverTimeQuery.isError"
              class="mb-4"
              color="error"
              variant="subtle"
              :title="balanceOverTimeQuery.error.message"
            />

            <div v-if="!balanceOverTimeQuery.data?.length" class="h-[300px] flex items-center justify-center text-muted">
              <div class="inline-flex items-center gap-2">
                <UIcon
                  v-if="balanceOverTimeQuery.isFetching"
                  name="i-lucide-loader-2"
                  class="size-4 animate-spin text-neutral-400"
                />
                <span v-else>No data</span>
              </div>
            </div>

            <VChart
              v-else
              :key="`${balanceOverTimePeriod}:${balanceOverTimeQuery.data?.length ?? 0}`"
              :option="balanceOverTimeOption"
              autoresize
              style="height: 300px; width: 100%"
            />
          </UPageCard>

          <UPageCard
            :ui="{ body: 'w-full' }"
          >
            <template #body>
              <div class="flex flex-row items-center justify-between">
                <div>
                  <div class="text-base text-pretty font-semibold text-highlighted">
                    Balances
                  </div>
                  <div class="text-[15px] text-pretty text-muted mt-1">
                    {{ balancesDescription }}
                  </div>
                </div>
                <div class="flex flex-row flex-wrap gap-3 items-end">
                  <UFormField name="viewType" label="View" :ui="{ label: 'text-muted' }">
                    <UTabs
                      v-model="tableView"
                      :items="tableViewItems"
                      :content="false"
                      color="neutral"
                      size="sm"
                    />
                  </UFormField>
                  <div class="flex items-center gap-2">
                    <UFormField name="groupBy" label="Group by" :ui="{ label: 'text-muted' }">
                      <USelect
                        v-model="balanceGroupBy"
                        :items="balanceGroupByItems"
                        class="w-44"
                        color="neutral"
                        variant="subtle"
                        :ui="{ base: 'min-h-9 ring-0' }"
                      />
                    </UFormField>
                  </div>
                  <div v-if="tableView === 'snapshots'" class="flex items-center gap-2">
                    <UButton
                      icon="i-lucide-plus"
                      :disabled="!accountQuery.data"
                      @click="addSnapshotsOpen = true"
                    >
                      Add snapshots
                    </UButton>
                    <UButton
                      v-if="selectedSnapshotCount"
                      color="error"
                      variant="subtle"
                      icon="i-lucide-trash-2"
                      @click="openSnapshotDeleteDialog(selectedSnapshotIds)"
                    >
                      Delete selected ({{ selectedSnapshotCount }})
                    </UButton>
                  </div>
                </div>
              </div>

              <UAlert
                v-if="snapshotsQuery.isError"
                class="mb-4"
                color="error"
                variant="subtle"
                :title="snapshotsQuery.error.message"
              />

              <UTable
                v-model:sorting="tableSorting"
                v-model:column-visibility="balanceColumnVisibility"
                :data="balanceTableData"
                :columns="balanceTableColumns"
                :grouping="balanceGrouping"
                :grouping-options="balanceGroupingOptions"
                :empty="balanceTableEmpty"
                :ui="{ root: 'min-w-full', td: 'empty:p-0' }"
                sticky
                class="max-h-[500px] overflow-auto"
              >
                <template #date-cell="{ row }">
                  <div class="flex items-center gap-2">
                    <span
                      class="inline-block"
                      :style="{ width: `calc(${row.depth} * 1rem)` }"
                    />

                    <template v-if="row.getIsGrouped()">
                      <UButton
                        variant="outline"
                        color="neutral"
                        size="xs"
                        class="shrink-0"
                        :icon="row.getIsExpanded() ? 'i-lucide-minus' : 'i-lucide-plus'"
                        :class="!row.getCanExpand() ? 'invisible' : ''"
                        :ui="{ base: 'p-0 rounded-sm', leadingIcon: 'size-4' }"
                        @click.stop="row.toggleExpanded()"
                      />

                      <span class="font-semibold text-highlighted">
                        {{ balanceGroupLabel(row) }}
                      </span>
                      <UBadge v-if="tableView === 'snapshots'" variant="subtle" color="neutral" size="sm">
                        {{ row.subRows.length }}
                      </UBadge>
                    </template>

                    <span v-else class="text-highlighted">
                      {{ formatShortDate(row.original.date) }}
                    </span>
                  </div>
                </template>

                <template #balance-cell="{ row }">
                  <span v-if="row.getIsGrouped()" class="font-semibold text-highlighted">
                    <span v-if="!groupedEndBalanceMinor(row)" class="text-muted">—</span>
                    <span v-else>{{ formatCurrencyMinor(groupedEndBalanceMinor(row)!, accountQuery.data.currency_code) }}</span>
                  </span>
                  <span v-else>
                    {{ formatCurrencyMinor(row.original.balance_minor, accountQuery.data.currency_code) }}
                  </span>
                </template>

                <template #change-cell="{ row }">
                  <template v-if="row.getIsGrouped()">
                    <span v-if="!groupedChangeMinor(row)" class="text-muted">—</span>
                    <span v-else :class="groupedChangeMinor(row)! >= 0 ? 'text-success' : 'text-error'">
                      {{ formatCurrencyMinor(groupedChangeMinor(row)!, accountQuery.data.currency_code, { signDisplay: "always" }) }}
                    </span>
                  </template>

                  <template v-else>
                    <span v-if="!row.original.change_minor" class="text-muted">—</span>
                    <span v-else :class="row.original.change_minor >= 0 ? 'text-success' : 'text-error'">
                      {{ formatCurrencyMinor(row.original.change_minor, accountQuery.data.currency_code, { signDisplay: "always" }) }}
                    </span>
                  </template>
                </template>
              </UTable>

              <div v-if="tableView === 'snapshots' && selectedSnapshotCount" class="mt-3 text-sm text-muted">
                {{ selectedSnapshotCount }} snapshot{{ selectedSnapshotCount === 1 ? '' : 's' }} selected
              </div>
            </template>
          </UPageCard>
        </template>

        <AccountsSnapshotsAddDialog
          v-model:open="addSnapshotsOpen"
          :account-id="accountId"
          :currency-code="accountCurrencyCode"
          :snapshots="snapshotsQuery.data ?? []"
        />

        <AccountsSnapshotEditDialog
          v-model:open="editSnapshotOpen"
          :account-id="accountId"
          :snapshot-id="editSnapshotId"
          :currency-code="accountCurrencyCode"
          :snapshots="snapshotsQuery.data ?? []"
        />

        <AccountsSnapshotsDeleteDialog
          v-model:open="deleteSnapshotsOpen"
          :account-id="accountId"
          :currency-code="accountCurrencyCode"
          :snapshots="deleteDialogSnapshots"
        />
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem, SelectItem, TableColumn, TableRow, TabsItem } from "@nuxt/ui";
import type { Column, GroupingOptions } from "@tanstack/vue-table";
import type { AccountBalanceSnapshotDto, BalanceOverTimePeriod, BalancePointDto } from "~/generated/bindings";
import type { AccountBreadcrumbContext } from "~/middleware/accountBreadcrumbContext.global";
import { useQuery } from "@tanstack/vue-query";
import { getGroupedRowModel } from "@tanstack/vue-table";
import { h, resolveComponent } from "vue";
import { useLocaleFormatters } from "~/composables/useLocaleFormatters";

interface BalanceRowBase {
  date: string
  balance_minor: number
  change_minor: number | null
}

interface SnapshotBalanceRow extends BalanceRowBase {
  kind: "snapshot"
  snapshot_id: number
  created_at: string
}

interface DailyBalanceRow extends BalanceRowBase {
  kind: "daily"
}

type BalanceRow = SnapshotBalanceRow | DailyBalanceRow;

type BalanceGroupBy = "none" | "month" | "year";

const darkTooltipBase = {
  backgroundColor: "rgba(10, 10, 10, 0.95)",
  borderColor: "rgba(82, 82, 91, 0.6)",
  borderWidth: 1,
  textStyle: { color: "#f5f5f5" },
  extraCssText: "border-radius: 10px; box-shadow: 0 10px 30px rgba(0,0,0,0.45); padding: 10px 12px;"
} as const;

const route = useRoute("accounts-id");
const api = useApi();
const accountBreadcrumbContext = useState<AccountBreadcrumbContext | null>("accountBreadcrumbContext", () => null);
const { formatCurrency, formatCurrencyMinor, formatDate, formatShortDate } = useLocaleFormatters();

const UButton = resolveComponent("UButton");
const UCheckbox = resolveComponent("UCheckbox");
const UDropdownMenu = resolveComponent("UDropdownMenu");

const accountId = useRouteParamInt(route, "id");

const accountQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.get(accountId.value!)),
  enabled: computed(() => accountId.value !== null),
  queryFn: async () => api.accountsGet(accountId.value!)
}));

const accountCurrencyCode = computed(() => accountQuery.data?.currency_code ?? "GBP");
const addSnapshotsOpen = ref(false);
const editSnapshotOpen = ref(false);
const editSnapshotId = ref<number | null>(null);
const deleteSnapshotsOpen = ref(false);
const deleteSnapshotIds = ref<number[]>([]);
const selectedSnapshotRowIds = ref<Record<string, boolean>>({});

useResourcePageError({
  resourceName: "Account",
  resourceId: accountId,
  resourceIsError: computed(() => accountQuery.isError),
  resourceError: computed(() => accountQuery.error),
  fallbackErrorMessage: "Failed to load account"
});

const snapshotsQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.snapshots(accountId.value!)),
  enabled: computed(() => accountQuery.isSuccess),
  queryFn: async () => api.accountSnapshotsList(accountId.value!)
}));

const balanceOverTimePeriod = ref<BalanceOverTimePeriod>("6M");

const balanceOverTimeQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.accounts.balanceOverTime(accountId.value!, balanceOverTimePeriod.value)),
  enabled: computed(() => accountQuery.isSuccess),
  queryFn: async () => api.accountBalanceOverTime(accountId.value!, balanceOverTimePeriod.value)
}));

const balanceOverTimePeriodItems = computed<TabsItem[]>(() => {
  const disabled = balanceOverTimeQuery.isFetching;
  return [
    { label: "1M", value: "1M", disabled },
    { label: "6M", value: "6M", disabled },
    { label: "1Y", value: "1Y", disabled },
    { label: "MAX", value: "MAX", disabled }
  ];
});

const breadcrumbItems = computed<BreadcrumbItem[]>(() => {
  const account = accountQuery.data;
  if (!account) return [];

  const context = accountBreadcrumbContext.value;
  if (context && context.accountId === account.id && context.institutionId === account.institution.id) {
    return [
      { label: "Institutions", to: { name: "institutions" }, icon: "i-lucide-building-2" },
      { label: account.institution.name, to: { name: "institutions-id", params: { id: account.institution.id } } },
      { label: account.name }
    ];
  }

  return [
    { label: "Accounts", to: { name: "accounts" }, icon: "i-lucide-wallet" },
    { label: account.name }
  ];
});

const headerDescription = computed(() => {
  const account = accountQuery.data;
  if (!account) return "";
  return `${account.institution.name} • ${ACCOUNT_TYPE_META[account.account_type.name].label}`;
});

function isCreatedAtAfter(a: string, b: string) {
  const ams = Date.parse(a);
  const bms = Date.parse(b);
  if (Number.isFinite(ams) && Number.isFinite(bms)) return ams > bms;
  return a > b;
}

const chartMeta = computed(() => {
  const kind = accountQuery.data?.account_type.name;
  if (!kind) {
    return {
      color: "#22c55e",
      glow: "rgba(34, 197, 94, 0.55)",
      glowEmphasis: "rgba(34, 197, 94, 0.85)"
    };
  }
  return ACCOUNT_TYPE_META[kind];
});

const monthlyChangeMinor = computed(() => {
  const points = balanceOverTimeQuery.data ?? [];
  if (points.length < 2) return null;

  const last = points.at(-1)?.balance_minor;
  const monthAgo = points.at(-31)?.balance_minor;
  const first = points[0]?.balance_minor;

  if (typeof last !== "number") return null;
  let change = null;
  if (typeof monthAgo === "number") change = last - monthAgo;
  else if (typeof first === "number") change = last - first;

  if (change === 0) return null;
  return change;
});

const balanceOverTimeOption = computed<ECOption>(() => {
  const points = balanceOverTimeQuery.data ?? [];
  const dates = points.map((p) => p.date);
  const values = points.map((p) => convertCurrencyMinorUnitsToMajorAmount(p.balance_minor));

  const invertAreaGradient = (() => {
    if (accountQuery.data?.normal_balance_sign !== -1) return false;
    if (!points.length) return false;

    let min = Infinity;
    let max = -Infinity;
    for (const p of points) {
      min = Math.min(min, p.balance_minor);
      max = Math.max(max, p.balance_minor);
    }

    if (!Number.isFinite(min) || !Number.isFinite(max)) return false;
    return max <= 0 && min < 0;
  })();

  const labelInterval = (idx: number, value: string) => {
    if (idx === 0 || idx === dates.length - 1) return true;

    const parts = value.split("-");
    const y = Number(parts[0] ?? 0);
    const m = Number(parts[1] ?? 0);
    const d = Number(parts[2] ?? 0);
    if (!Number.isFinite(y) || !Number.isFinite(m) || !Number.isFinite(d)) return false;

    switch (balanceOverTimePeriod.value) {
    case "1M":
      return d === 1 || d === 8 || d === 15 || d === 22 || d === 29;
    case "6M":
      return d === 15;
    case "1Y":
      return d === 15 && (m % 2 === 1);
    case "MAX":
      return m === 1 && d === 1;
    }
  };

  const labelFormatter = (value: string) => {
    const parts = value.split("-");
    const y = Number(parts[0] ?? 0);
    const m = Number(parts[1] ?? 0);
    const d = Number(parts[2] ?? 0);
    if (!Number.isFinite(y) || !Number.isFinite(m) || !Number.isFinite(d)) return value;

    switch (balanceOverTimePeriod.value) {
    case "1M":
      return formatDate(value, { day: "numeric", month: "short" }, value);
    case "6M":
    case "1Y":
      return formatDate(value, { month: "short" }, value);
    case "MAX":
      return formatDate(value, { year: "numeric" }, value);
    }
  };

  return {
    backgroundColor: "transparent",
    tooltip: {
      ...darkTooltipBase,
      trigger: "axis",
      axisPointer: {
        type: "line",
        lineStyle: { color: "rgba(244, 244, 245, 0.25)" }
      },
      valueFormatter: (value: unknown) => {
        const n = typeof value === "number" ? value : Number(value);
        if (!Number.isFinite(n)) return String(value);
        return formatCurrency(n, accountCurrencyCode.value);
      }
    },
    grid: {
      left: 0,
      right: 0,
      top: 0,
      bottom: 0,
      containLabel: true
    },
    xAxis: {
      type: "category",
      data: dates,
      boundaryGap: false,
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { show: false },
      axisLabel: {
        hideOverlap: true,
        interval: labelInterval,
        formatter: labelFormatter
      }
    },
    yAxis: {
      type: "value",
      show: true,
      scale: true,
      axisTick: { show: false },
      axisLine: {
        show: false
      },
      axisLabel: {
        color: "#a3a3a3",
        formatter: (value: number) => {
          return formatCurrency(value, accountCurrencyCode.value);
        }
      },
      splitLine: {
        show: true,
        lineStyle: { color: "rgba(244, 244, 245, 0.06)" }
      },
      minInterval: 50
    },
    series: [
      {
        name: "Balance",
        type: "line",
        data: values,
        smooth: true,
        showSymbol: false,
        lineStyle: {
          width: 5,
          color: chartMeta.value.color,
          shadowBlur: 10,
          shadowColor: chartMeta.value.glow,
          shadowOffsetX: 0,
          shadowOffsetY: 0
        },
        itemStyle: { color: chartMeta.value.color },
        areaStyle: {
          color: {
            type: "linear",
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: invertAreaGradient ? "rgba(0, 0, 0, 0)" : chartMeta.value.glowEmphasis },
              { offset: 1, color: invertAreaGradient ? chartMeta.value.glowEmphasis : "rgba(0, 0, 0, 0)" }
            ]
          }
        },
        emphasis: {
          focus: "series",
          lineStyle: {
            shadowBlur: 14,
            shadowColor: chartMeta.value.glowEmphasis
          }
        }
      }
    ]
  };
});

const tableView = ref<"snapshots" | "daily">("snapshots");
const tableViewItems = ref<TabsItem[]>([
  { label: "Snapshots", value: "snapshots" },
  { label: "Daily", value: "daily" }
]);

const selectedSnapshotIds = computed(() => {
  return Object.entries(selectedSnapshotRowIds.value)
    .filter(([, selected]) => selected)
    .map(([snapshotId]) => Number.parseInt(snapshotId, 10))
    .filter((snapshotId) => Number.isFinite(snapshotId));
});

const selectedSnapshotCount = computed(() => selectedSnapshotIds.value.length);
const allSnapshotsSelected = computed(() => {
  const totalSnapshots = (snapshotsQuery.data ?? []).length;
  return totalSnapshots > 0 && selectedSnapshotCount.value === totalSnapshots;
});
const someSnapshotsSelected = computed(() => {
  return selectedSnapshotCount.value > 0 && !allSnapshotsSelected.value;
});

const deleteDialogSnapshots = computed<AccountBalanceSnapshotDto[]>(() => {
  const byId = new Map((snapshotsQuery.data ?? []).map((snapshot) => [snapshot.id, snapshot]));
  return deleteSnapshotIds.value
    .map((snapshotId) => byId.get(snapshotId) ?? null)
    .filter((snapshot): snapshot is AccountBalanceSnapshotDto => snapshot != null)
    .sort((left, right) => left.date.localeCompare(right.date));
});

watch([() => snapshotsQuery.data, tableView], () => {
  if (tableView.value !== "snapshots") {
    selectedSnapshotRowIds.value = {};
    return;
  }

  const validIds = new Set((snapshotsQuery.data ?? []).map((snapshot) => String(snapshot.id)));
  selectedSnapshotRowIds.value = Object.fromEntries(
    Object.entries(selectedSnapshotRowIds.value).filter(([snapshotId, selected]) => selected && validIds.has(snapshotId))
  );
}, { immediate: true });

function isSnapshotSelected(snapshotId: number) {
  return !!selectedSnapshotRowIds.value[String(snapshotId)];
}

function setSnapshotSelected(snapshotId: number, selected: boolean) {
  const key = String(snapshotId);
  if (selected) {
    selectedSnapshotRowIds.value = { ...selectedSnapshotRowIds.value, [key]: true };
    return;
  }

  const next = { ...selectedSnapshotRowIds.value };
  delete next[key];
  selectedSnapshotRowIds.value = next;
}

function toggleAllSnapshots(selected: boolean) {
  if (!selected) {
    selectedSnapshotRowIds.value = {};
    return;
  }

  selectedSnapshotRowIds.value = Object.fromEntries(
    (snapshotsQuery.data ?? []).map((snapshot) => [String(snapshot.id), true])
  );
}

function groupedSnapshotIds(row: TableRow<BalanceRow>) {
  return row.getLeafRows()
    .map((leafRow) => leafRow.original)
    .filter(isSnapshotBalanceRow)
    .map((snapshotRow) => snapshotRow.snapshot_id);
}

function groupedSnapshotSelectionState(row: TableRow<BalanceRow>) {
  const snapshotIds = groupedSnapshotIds(row);
  if (!snapshotIds.length) return false;

  const selectedCount = snapshotIds.filter((snapshotId) => isSnapshotSelected(snapshotId)).length;
  if (selectedCount === 0) return false;
  if (selectedCount === snapshotIds.length) return true;
  return "indeterminate" as const;
}

function setGroupedSnapshotsSelected(row: TableRow<BalanceRow>, selected: boolean) {
  const snapshotIds = groupedSnapshotIds(row);
  if (!snapshotIds.length) return;

  const next = { ...selectedSnapshotRowIds.value };
  for (const snapshotId of snapshotIds) {
    const key = String(snapshotId);
    if (selected) next[key] = true;
    else delete next[key];
  }
  selectedSnapshotRowIds.value = next;
}

function openSnapshotEditor(snapshotId: number) {
  editSnapshotId.value = snapshotId;
  editSnapshotOpen.value = true;
}

function openSnapshotDeleteDialog(snapshotIds: number[]) {
  deleteSnapshotIds.value = [...new Set(snapshotIds)];
  deleteSnapshotsOpen.value = deleteSnapshotIds.value.length > 0;
}

const balanceGroupByItems = ref<SelectItem[]>([
  { label: "None", value: "none" },
  { label: "Month", value: "month" },
  { label: "Year", value: "year" }
]);
const balanceGroupBy = ref<BalanceGroupBy>("none");

const derivedDailyPoints = computed<BalancePointDto[]>(() => {
  const snaps = snapshotsQuery.data ?? [];
  if (!snaps.length) return [];

  // Pick one balance per day; if multiple snapshots share a date, keep the latest created_at.
  const byDate = new Map<string, { balance_minor: number, created_at: string }>();
  let minDate: string | null = null;
  let maxDate: string | null = null;

  for (const s of snaps) {
    const date = String(s.date ?? "");
    if (!date) continue;

    if (minDate == null || date < minDate) minDate = date;
    if (maxDate == null || date > maxDate) maxDate = date;

    const existing = byDate.get(date);
    if (!existing) {
      byDate.set(date, { balance_minor: s.balance_minor, created_at: s.created_at });
      continue;
    }
    if (isCreatedAtAfter(String(s.created_at ?? ""), existing.created_at)) {
      byDate.set(date, { balance_minor: s.balance_minor, created_at: s.created_at });
    }
  }

  if (minDate == null) return [];

  const today = getTodayCalendarDateIsoString();
  let endDate = today;
  if (maxDate != null && maxDate > endDate) endDate = maxDate;

  const startMs = getUtcMillisecondsFromCalendarDateIsoString(minDate);
  const endMs = getUtcMillisecondsFromCalendarDateIsoString(endDate);
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs < startMs) return [];

  const out: BalancePointDto[] = [];
  let last: number | null = null;
  for (let ms = startMs; ms <= endMs; ms += 86400000) {
    const iso = getCalendarDateIsoStringFromUtcMilliseconds(ms);
    const v = byDate.get(iso)?.balance_minor;
    if (typeof v === "number") last = v;
    out.push({ date: iso, balance_minor: last ?? 0 });
  }

  return out;
});

const balancesDescription = computed(() => {
  if (tableView.value === "snapshots") {
    return "All stored balance snapshots";
  }
  return "Daily balances (all time)";
});

const balanceGrouping = computed(() => {
  if (balanceGroupBy.value === "month") {
    return ["month_group"];
  }
  if (balanceGroupBy.value === "year") {
    return ["year_group"];
  }
  return [];
});

const balanceGroupingOptions = ref<GroupingOptions>({
  groupedColumnMode: "remove",
  getGroupedRowModel: getGroupedRowModel()
});

const balanceColumnVisibility = ref<Record<string, boolean>>({
  month_group: false,
  year_group: false
});

const snapshotTableRows = computed<SnapshotBalanceRow[]>(() => {
  const snaps = snapshotsQuery.data ?? [];
  // Backend returns DESC by date; compute change vs previous snapshot in time (older snapshot).
  return snaps.map((s, idx) => {
    const next = snaps[idx + 1];
    const change_minor = next ? (s.balance_minor - next.balance_minor) : null;
    return {
      kind: "snapshot",
      snapshot_id: s.id,
      created_at: s.created_at,
      date: s.date,
      balance_minor: s.balance_minor,
      change_minor
    };
  });
});

const dailyTableRows = computed<DailyBalanceRow[]>(() => {
  const points = derivedDailyPoints.value ?? [];
  if (!points.length) return [];

  const out: DailyBalanceRow[] = [];
  for (let i = points.length - 1; i >= 0; i--) {
    const p = points[i]!;
    const prev = i > 0 ? points[i - 1] : undefined;
    const change_minor = prev ? (p.balance_minor - prev.balance_minor) : null;
    out.push({
      kind: "daily",
      date: p.date,
      balance_minor: p.balance_minor,
      change_minor
    });
  }

  return out;
});

const balanceTableData = computed<BalanceRow[]>(() => {
  if (tableView.value === "snapshots") {
    return snapshotTableRows.value;
  }
  return dailyTableRows.value;
});

const balanceTableEmpty = computed(() => {
  if (tableView.value === "snapshots") {
    return snapshotsQuery.isFetching ? "" : "No snapshots yet.";
  }
  return snapshotsQuery.isFetching ? "" : "No data.";
});

const balanceGroupStatsByKey = computed(() => {
  const mode = balanceGroupBy.value;
  if (mode === "none") {
    return new Map<string, { endBalance_minor: number, changeFromPrevGroup_minor: number | null }>();
  }

  const rows = balanceTableData.value;
  if (!rows.length) {
    return new Map<string, { endBalance_minor: number, changeFromPrevGroup_minor: number | null }>();
  }

  const keyOf = (r: BalanceRow) => (mode === "month" ? r.date.slice(0, 7) : r.date.slice(0, 4));

  // Iterate oldest -> newest (table rows are newest -> oldest).
  const endBalanceByKey = new Map<string, number>();
  const order: string[] = [];

  for (let i = rows.length - 1; i >= 0; i--) {
    const r = rows[i]!;
    const key = keyOf(r);
    if (!endBalanceByKey.has(key)) {
      order.push(key);
    }
    // As we walk forward in time, this ends up being the latest balance in the group.
    endBalanceByKey.set(key, r.balance_minor);
  }

  const out = new Map<string, { endBalance_minor: number, changeFromPrevGroup_minor: number | null }>();
  let prevEnd: number | null = null;

  for (const key of order) {
    const endBalance_minor = endBalanceByKey.get(key)!;
    const changeFromPrevGroup_minor = prevEnd == null ? null : (endBalance_minor - prevEnd);
    out.set(key, { endBalance_minor, changeFromPrevGroup_minor });
    prevEnd = endBalance_minor;
  }

  return out;
});

function groupKeyFromTableRow(row: TableRow<BalanceRow>) {
  const id = row.groupingColumnId;
  if (id == null) return null;
  const key = row.getValue<string>(id);
  if (!key) return null;
  return { id, key };
}

function balanceGroupLabel(row: TableRow<BalanceRow>) {
  const k = groupKeyFromTableRow(row);
  if (!k) return "";

  if (k.id === "month_group") {
    const [y, m] = k.key.split("-");
    const year = Number(y);
    const month = Number(m);

    if (!Number.isFinite(year) || !Number.isFinite(month) || month < 1 || month > 12) {
      return k.key;
    }

    return formatDate(`${k.key}-01`, { month: "short", year: "numeric" }, k.key);
  }
  return k.key;
}

function groupedEndBalanceMinor(row: TableRow<BalanceRow>) {
  const k = groupKeyFromTableRow(row);
  if (!k) return null;
  return balanceGroupStatsByKey.value.get(k.key)?.endBalance_minor ?? null;
}

function groupedChangeMinor(row: TableRow<BalanceRow>) {
  const k = groupKeyFromTableRow(row);
  if (!k) return null;
  return balanceGroupStatsByKey.value.get(k.key)?.changeFromPrevGroup_minor ?? null;
}

function isSnapshotBalanceRow(row: BalanceRow): row is SnapshotBalanceRow {
  return row.kind === "snapshot";
}

const tableSorting = ref([
  {
    id: "date",
    desc: true
  }
]);

function sortableHeader(column: Column<BalanceRow, unknown>, label: string) {
  const isSorted = column.getIsSorted();

  return h(UButton, {
    color: "neutral",
    variant: "ghost",
    label,
    trailing: isSorted !== false,
    trailingIcon: isSorted !== false
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

function snapshotRowActions(row: SnapshotBalanceRow) {
  return [
    {
      type: "label" as const,
      label: "Actions"
    },
    {
      label: "Edit snapshot",
      icon: "i-lucide-pencil",
      onSelect: () => openSnapshotEditor(row.snapshot_id)
    },
    {
      label: "Delete snapshot",
      icon: "i-lucide-trash-2",
      color: "error" as const,
      onSelect: () => openSnapshotDeleteDialog([row.snapshot_id])
    }
  ];
}

const balanceTableColumns = computed<TableColumn<BalanceRow>[]>(() => [
  {
    id: "month_group",
    accessorFn: (row) => row.date.slice(0, 7),
    enableSorting: false
  },
  {
    id: "year_group",
    accessorFn: (row) => row.date.slice(0, 4),
    enableSorting: false
  },
  ...(tableView.value === "snapshots"
    ? [{
      id: "select",
      enableSorting: false,
      meta: {
        class: {
          th: "w-auto",
          td: "w-auto"
        }
      },
      header: () => h(UCheckbox, {
        modelValue: someSnapshotsSelected.value ? "indeterminate" : allSnapshotsSelected.value,
        "onUpdate:modelValue": (value: boolean | "indeterminate") => toggleAllSnapshots(value === true),
        "aria-label": "Select all snapshots"
      }),
      cell: ({ row }: { row: TableRow<BalanceRow> }) => {
        if (row.getIsGrouped()) {
          return h(UCheckbox, {
            modelValue: groupedSnapshotSelectionState(row),
            "onUpdate:modelValue": (value: boolean | "indeterminate") => setGroupedSnapshotsSelected(row, value === true),
            "aria-label": "Select grouped snapshots"
          });
        }

        const originalRow = row.original;
        if (!isSnapshotBalanceRow(originalRow)) return null;

        return h(UCheckbox, {
          modelValue: isSnapshotSelected(originalRow.snapshot_id),
          "onUpdate:modelValue": (value: boolean | "indeterminate") => setSnapshotSelected(originalRow.snapshot_id, value === true),
          "aria-label": `Select snapshot ${originalRow.snapshot_id}`
        });
      }
    } satisfies TableColumn<BalanceRow>]
    : []),
  {
    accessorKey: "date",
    header: ({ column }) => sortableHeader(column, "Date"),
    meta: {
      class: {
        th: "w-full",
        td: "w-full"
      }
    }
  },
  {
    id: "balance",
    accessorKey: "balance_minor",
    header: ({ column }) => sortableHeader(column, "Balance"),
    meta: {
      class: {
        th: "w-auto",
        td: "text-right w-auto"
      }
    }
  },
  {
    id: "change",
    accessorKey: "change_minor",
    header: ({ column }) => sortableHeader(column, "Change"),
    meta: {
      class: {
        th: "w-auto",
        td: "text-right w-auto"
      }
    }
  },
  ...(tableView.value === "snapshots"
    ? [{
      id: "actions",
      enableSorting: false,
      meta: {
        class: {
          th: "w-auto",
          td: "text-right w-auto"
        }
      },
      header: "",
      cell: ({ row }: { row: TableRow<BalanceRow> }) => {
        const originalRow = row.original;
        if (row.getIsGrouped() || !isSnapshotBalanceRow(originalRow)) return null;

        return h(UDropdownMenu, {
          items: snapshotRowActions(originalRow),
          content: { align: "end" },
          "aria-label": "Snapshot actions"
        }, () => h(UButton, {
          icon: "i-lucide-ellipsis-vertical",
          color: "neutral",
          variant: "ghost",
          "aria-label": "Snapshot actions"
        }));
      }
    } satisfies TableColumn<BalanceRow>]
    : [])
]);
</script>
