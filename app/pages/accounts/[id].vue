<template>
  <UContainer>
    <div v-if="accountQuery.data" class="pt-6">
      <UBreadcrumb :items="breadcrumbItems" />
    </div>

    <UPageHeader
      v-if="accountQuery.data"
      :title="accountQuery.data.name"
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
        title="Invalid account id"
      />

      <UAlert
        v-else-if="accountQuery.isError"
        color="error"
        variant="subtle"
        :title="accountQuery.error!.message ?? 'Failed to load account'"
      />

      <template v-else-if="accountQuery.data">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <UPageCard
            title="Current Balance"
            variant="subtle"
            :ui="{ title: 'text-muted text-xs whitespace-nowrap' }"
          >
            <template #description>
              <div class="text-xl font-bold text-default whitespace-nowrap">
                {{ formatMoneyMinor(accountQuery.data.latest_balance_minor) }}
              </div>
              <div class="text-xs text-muted mt-1">
                As of {{ formatShortDate(accountQuery.data.latest_snapshot_date) }}
              </div>
            </template>
          </UPageCard>

          <UPageCard
            title="Institution"
            :description="accountQuery.data.institution.name"
            variant="subtle"
            :ui="{
              title: 'text-muted text-xs whitespace-nowrap',
              description: 'text-xl font-bold text-default whitespace-nowrap'
            }"
          />

          <UPageCard
            title="Account Type"
            variant="subtle"
            :ui="{ title: 'text-muted text-xs whitespace-nowrap' }"
          >
            <template #description>
              <div class="flex items-center gap-2">
                <UBadge
                  variant="subtle"
                  color="neutral"
                  size="xl"
                  :class="accountTypeBadgeClass(accountQuery.data.account_type.name)"
                >
                  {{ accountTypeLabel(accountQuery.data.account_type.name) }}
                </UBadge>
              </div>
            </template>
          </UPageCard>

          <UPageCard
            title="Monthly Change"
            variant="subtle"
            :ui="{
              title: 'text-muted text-xs whitespace-nowrap'
            }"
          >
            <template #description>
              <div class="flex items-center gap-2 whitespace-nowrap">
                <UIcon :name="monthlyChangeIcon" class="size-4" :class="monthlyChangeClass" />
                <span class="text-xl font-bold" :class="monthlyChangeClass">
                  {{ monthlyChangeLabel }}
                </span>
              </div>
              <div class="text-xs text-muted mt-1">
                Change over 30 days
              </div>
            </template>
          </UPageCard>
        </div>

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
            :title="balanceOverTimeQuery.error!.message ?? 'Failed to load balance series'"
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
          title="Balances"
          :description="balancesDescription"
          :ui="{
            title: 'text-highlighted',
            description: 'mt-1',
            body: 'min-w-full'
          }"
        >
          <template #body>
            <div class="flex items-center justify-between gap-4 mb-4">
              <UTabs
                v-model="tableView"
                :items="tableViewItems"
                :content="false"
                color="neutral"
                size="sm"
              />

              <div class="flex items-center gap-2">
                <div class="text-sm text-muted">
                  Group by
                </div>
                <USelect
                  v-model="balanceGroupBy"
                  :items="balanceGroupByItems"
                  class="w-44"
                  color="neutral"
                  variant="subtle"
                  :content="{ bodyLock: false }"
                />
              </div>
            </div>

            <UAlert
              v-if="snapshotsQuery.isError"
              class="mb-4"
              color="error"
              variant="subtle"
              :title="snapshotsQuery.error!.message ?? 'Failed to load balances'"
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
                      {{ row.subRows?.length || 0 }}
                    </UBadge>
                  </template>

                  <span v-else class="font-medium text-highlighted">
                    {{ formatShortDate(row.original.date) }}
                  </span>
                </div>
              </template>

              <template #balance-cell="{ row }">
                <span v-if="row.getIsGrouped()" class="font-semibold text-highlighted">
                  <span v-if="!groupedEndBalanceMinor(row)" class="text-muted">—</span>
                  <span v-else>{{ formatMoneyMinor(groupedEndBalanceMinor(row)!) }}</span>
                </span>
                <span v-else class="font-medium">
                  {{ formatMoneyMinor(row.original.balance_minor) }}
                </span>
              </template>

              <template #change-cell="{ row }">
                <template v-if="row.getIsGrouped()">
                  <span v-if="!groupedChangeMinor(row)" class="text-muted">—</span>
                  <span v-else :class="groupedChangeMinor(row)! >= 0 ? 'text-success' : 'text-error'">
                    {{ formatSignedMoneyMinor(groupedChangeMinor(row)!) }}
                  </span>
                </template>

                <template v-else>
                  <span v-if="!row.original.change_minor" class="text-muted">—</span>
                  <span v-else :class="row.original.change_minor >= 0 ? 'text-success' : 'text-error'">
                    {{ formatSignedMoneyMinor(row.original.change_minor) }}
                  </span>
                </template>
              </template>
            </UTable>
          </template>
        </UPageCard>
      </template>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BreadcrumbItem, SelectItem, TableColumn, TabsItem } from "@nuxt/ui";
import type { GroupingOptions } from "@tanstack/vue-table";
import type { AccountBalanceSnapshotDto, BalanceOverTimePeriod, BalancePointDto } from "~/bindings";

import { useQuery } from "@tanstack/vue-query";
import { getGroupedRowModel } from "@tanstack/vue-table";
import { computed, h, proxyRefs, resolveComponent } from "vue";

import { ACCOUNT_TYPE_META, accountTypeBadgeClass, accountTypeLabel } from "~/utils/account-type-meta";

interface BalanceRow {
  date: string
  balance_minor: number
  change_minor: number | null
}

type BalanceGroupBy = "none" | "month" | "year";

const monthShort = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;

const darkTooltipBase = {
  backgroundColor: "rgba(10, 10, 10, 0.95)",
  borderColor: "rgba(82, 82, 91, 0.6)",
  borderWidth: 1,
  textStyle: { color: "#f5f5f5" },
  extraCssText: "border-radius: 10px; box-shadow: 0 10px 30px rgba(0,0,0,0.45); padding: 10px 12px;"
} as const;

const route = useRoute();
const api = useApi();

const UButton = resolveComponent("UButton");

const rawId = computed(() => {
  const p = (route.params as any)?.id;
  if (Array.isArray(p)) return p[0];
  return p;
});

const accountId = computed<number | null>(() => {
  const s = String(rawId.value ?? "");
  const n = Number.parseInt(s, 10);
  if (!Number.isFinite(n)) return null;
  return n;
});

const invalidId = computed(() => rawId.value != null && accountId.value == null);

const accountQuery = proxyRefs(useQuery({
  queryKey: ["accounts", "get", accountId],
  enabled: computed(() => typeof accountId.value === "number"),
  queryFn: () => api.accountsGet(accountId.value as number)
}));

const snapshotsQuery = proxyRefs(useQuery({
  queryKey: ["accounts", "snapshots", accountId],
  enabled: computed(() => typeof accountId.value === "number"),
  queryFn: () => api.accountSnapshotsList(accountId.value as number)
}));

const balanceOverTimePeriod = ref<BalanceOverTimePeriod>("6M");

const balanceOverTimeQuery = proxyRefs(useQuery({
  queryKey: ["accounts", "balanceOverTime", accountId, balanceOverTimePeriod],
  enabled: computed(() => typeof accountId.value === "number"),
  queryFn: () => api.accountBalanceOverTime(accountId.value as number, balanceOverTimePeriod.value)
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
  return [
    { label: "Accounts", to: "/accounts", icon: "i-lucide-wallet" },
    { label: account?.name ?? "" }
  ];
});

const headerDescription = computed(() => {
  const account = accountQuery.data;
  if (!account) return "";
  return `${account.institution.name} • ${accountTypeLabel(account.account_type.name)}`;
});

const moneyFormatter = computed(() => {
  const code = accountQuery.data?.currency_code ?? "GBP";
  return new Intl.NumberFormat("en-GB", {
    style: "currency",
    currency: code
  });
});

function formatMoneyMinor(minor: number) {
  return moneyFormatter.value.format(minor / 100);
}

function formatSignedMoneyMinor(minor: number) {
  const sign = minor >= 0 ? "+" : "-";
  return `${sign}${formatMoneyMinor(Math.abs(minor))}`;
}

function parseIsoDate(iso: string) {
  // Avoid timezone shifts for date-only strings.
  return new Date(`${iso}T00:00:00`);
}

function utcTodayIsoDate() {
  return new Date().toISOString().slice(0, 10);
}

function isoDateToUtcMs(iso: string) {
  const parts = iso.split("-");
  if (parts.length !== 3) return Number.NaN;

  const y = Number.parseInt(parts[0] ?? "", 10);
  const m = Number.parseInt(parts[1] ?? "", 10);
  const d = Number.parseInt(parts[2] ?? "", 10);

  if (!Number.isFinite(y) || !Number.isFinite(m) || !Number.isFinite(d)) return Number.NaN;
  return Date.UTC(y, m - 1, d);
}

function utcMsToIsoDate(ms: number) {
  return new Date(ms).toISOString().slice(0, 10);
}

function isCreatedAtAfter(a: string, b: string) {
  const ams = Date.parse(a);
  const bms = Date.parse(b);
  if (Number.isFinite(ams) && Number.isFinite(bms)) return ams > bms;
  return a > b;
}

function formatShortDate(iso: string) {
  return parseIsoDate(iso).toLocaleDateString("en-GB", {
    day: "2-digit",
    month: "short",
    year: "numeric"
  });
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
  if (typeof monthAgo === "number") return last - monthAgo;
  if (typeof first === "number") return last - first;
  return null;
});

const monthlyChangeClass = computed(() => {
  if (monthlyChangeMinor.value == null) return "text-muted";
  return monthlyChangeMinor.value >= 0 ? "text-success" : "text-error";
});

const monthlyChangeIcon = computed(() => {
  if (monthlyChangeMinor.value == null) return "i-lucide-minus";
  return monthlyChangeMinor.value >= 0 ? "i-lucide-arrow-up" : "i-lucide-arrow-down";
});

const monthlyChangeLabel = computed(() => {
  if (monthlyChangeMinor.value == null) return "—";
  return formatSignedMoneyMinor(monthlyChangeMinor.value);
});

const balanceOverTimeOption = computed<ECOption>(() => {
  const points = balanceOverTimeQuery.data ?? [];
  const dates = points.map((p) => p.date);
  const values = points.map((p) => p.balance_minor / 100);

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
      return `${monthShort[m - 1] ?? ""} ${d}`.trim();
    case "6M":
    case "1Y":
      return monthShort[m - 1] ?? "";
    case "MAX":
      return String(y);
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
        // `n` is in major units (e.g. pounds) here.
        return moneyFormatter.value.format(n);
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
          return moneyFormatter.value.format(value);
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

  for (const s of snaps as AccountBalanceSnapshotDto[]) {
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

  const today = utcTodayIsoDate();
  let endDate = today;
  if (maxDate != null && maxDate > endDate) endDate = maxDate;

  const startMs = isoDateToUtcMs(minDate);
  const endMs = isoDateToUtcMs(endDate);
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs < startMs) return [];

  const out: BalancePointDto[] = [];
  let last: number | null = null;
  for (let ms = startMs; ms <= endMs; ms += 86400000) {
    const iso = utcMsToIsoDate(ms);
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

const snapshotTableRows = computed<BalanceRow[]>(() => {
  const snaps = snapshotsQuery.data ?? [];
  // Backend returns DESC by date; compute change vs previous snapshot in time (older snapshot).
  return snaps.map((s, idx) => {
    const next = snaps[idx + 1];
    const change_minor = next ? (s.balance_minor - next.balance_minor) : null;
    return {
      date: s.date,
      balance_minor: s.balance_minor,
      change_minor
    };
  });
});

const dailyTableRows = computed<BalanceRow[]>(() => {
  const points = derivedDailyPoints.value ?? [];
  if (!points.length) return [];

  const out: BalanceRow[] = [];
  for (let i = points.length - 1; i >= 0; i--) {
    const p = points[i]!;
    const prev = i > 0 ? points[i - 1] : undefined;
    const change_minor = prev ? (p.balance_minor - prev.balance_minor) : null;
    out.push({
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

function groupKeyFromTableRow(row: any) {
  const id = row?.groupingColumnId as string | undefined;
  if (!id) return null;
  const key = String(row.getValue(id) ?? "");
  if (!key) return null;
  return { id, key };
}

function balanceGroupLabel(row: any) {
  const k = groupKeyFromTableRow(row);
  if (!k) return "";

  if (k.id === "month_group") {
    const [y, m] = k.key.split("-");
    const year = Number.parseInt(String(y ?? ""), 10);
    const month = Number.parseInt(String(m ?? ""), 10);

    if (!Number.isFinite(year) || !Number.isFinite(month) || month < 1 || month > 12) {
      return k.key;
    }

    return `${monthShort[month - 1] ?? ""} ${year}`.trim();
  }
  return k.key;
}

function groupedEndBalanceMinor(row: any) {
  const k = groupKeyFromTableRow(row);
  if (!k) return null;
  return balanceGroupStatsByKey.value.get(k.key)?.endBalance_minor ?? null;
}

function groupedChangeMinor(row: any) {
  const k = groupKeyFromTableRow(row);
  if (!k) return null;
  return balanceGroupStatsByKey.value.get(k.key)?.changeFromPrevGroup_minor ?? null;
}

const tableSorting = ref([
  {
    id: "date",
    desc: true
  }
]);

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
  {
    accessorKey: "date",
    header: ({ column }) => sortableHeader(column, "Date")
  },
  {
    id: "balance",
    accessorKey: "balance_minor",
    header: ({ column }) => sortableHeader(column, "Balance"),
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  },
  {
    id: "change",
    accessorKey: "change_minor",
    header: ({ column }) => sortableHeader(column, "Change"),
    meta: {
      class: {
        th: "text-right",
        td: "text-right"
      }
    }
  }
]);
</script>
