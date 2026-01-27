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
                v-if="row.original.currentBalance === 0"
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
            {{ row.original.institution }}
          </span>
        </template>

        <template #type-cell="{ row }">
          <span v-if="row.getIsGrouped()" />
          <UBadge
            v-else
            variant="subtle"
            color="neutral"
            :class="accountTypeBadgeClass(row.original.type)"
          >
            {{ accountTypeLabel(row.original.type) }}
          </UBadge>
        </template>

        <template #firstChange-cell="{ row }">
          <span v-if="row.getIsGrouped()">
            {{ formatShortDayNumber(getGroupedFirstChange(row)) }}
          </span>
          <span v-else>
            {{ formatShortDayNumber(accountHistoryById[row.original.id]?.startDay) }}
          </span>
        </template>

        <template #lastChange-cell="{ row }">
          <span v-if="row.getIsGrouped()">
            {{ formatShortDate(getGroupedLastChange(row)) }}
          </span>
          <span v-else>
            {{ formatShortDate(row.original.lastBalanceChangeDate) }}
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
            {{ formatGBP(row.original.currentBalance) }}
          </span>
        </template>
      </UTable>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { SelectItem, TableColumn, TableRow } from "@nuxt/ui";
import type { GroupingOptions } from "@tanstack/vue-table";
import { getGroupedRowModel } from "@tanstack/vue-table";
import { h, resolveComponent } from "vue";

type AccountType
  = "current"
    | "savings"
    | "stocks"
    | "isa"
    | "pension";

type ActivityPeriod
  = "1W"
    | "1M"
    | "3M"
    | "6M";

interface Account {
  id: string
  name: string
  institution: string
  type: AccountType
  currentBalance: number
  lastBalanceChangeDate: string // YYYY-MM-DD
}

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

const rawAccounts = ref<Account[]>([
  {
    id: "acc_01",
    name: "Everyday Current",
    institution: "Nationwide",
    type: "current",
    currentBalance: 2435.12,
    lastBalanceChangeDate: "2026-01-21"
  },
  {
    id: "acc_02",
    name: "Bills Pot",
    institution: "Monzo",
    type: "current",
    currentBalance: 0,
    lastBalanceChangeDate: "2025-12-18"
  },
  {
    id: "acc_03",
    name: "Rainy Day Savings",
    institution: "Nationwide",
    type: "savings",
    currentBalance: 13250.0,
    lastBalanceChangeDate: "2026-01-12"
  },
  {
    id: "acc_04",
    name: "Emergency Fund",
    institution: "Nationwide",
    type: "savings",
    currentBalance: 8000.0,
    lastBalanceChangeDate: "2025-11-04"
  },
  {
    id: "acc_05",
    name: "Stocks & Shares ISA",
    institution: "Trading 212",
    type: "isa",
    currentBalance: 45890.42,
    lastBalanceChangeDate: "2026-01-23"
  },
  {
    id: "acc_06",
    name: "General Investment Account",
    institution: "Trading 212",
    type: "stocks",
    currentBalance: 12110.7,
    lastBalanceChangeDate: "2026-01-20"
  },
  {
    id: "acc_07",
    name: "Workplace Pension",
    institution: "Aviva",
    type: "pension",
    currentBalance: 98025.33,
    lastBalanceChangeDate: "2026-01-02"
  },
  {
    id: "acc_08",
    name: "Holiday Savings",
    institution: "Starling",
    type: "savings",
    currentBalance: 1420.0,
    lastBalanceChangeDate: "2025-12-29"
  },
  {
    id: "acc_09",
    name: "Cash ISA (Legacy)",
    institution: "HSBC",
    type: "isa",
    currentBalance: 0,
    lastBalanceChangeDate: "2024-08-12"
  }
]);

const accountsData = computed(() => {
  if (showEmpty.value) {
    return rawAccounts.value;
  }

  return rawAccounts.value.filter((a) => a.currentBalance !== 0);
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

const ACCOUNT_TYPE_LABEL: Record<AccountType, string> = {
  current: "Current",
  savings: "Savings",
  stocks: "Stocks",
  isa: "ISA",
  pension: "Pension"
};

function accountTypeLabel(type: AccountType) {
  return ACCOUNT_TYPE_LABEL[type] ?? type;
}

const ACCOUNT_TYPE_BADGE_CLASS: Record<AccountType, string> = {
  current: "bg-[#3B82F6]/20 text-[#93C5FD] ring-[#3B82F6]/45",
  savings: "bg-[#16A34A]/15 text-[#4ADE80] ring-[#16A34A]/35",
  stocks: "bg-[#7C3AED]/15 text-[#C4B5FD] ring-[#7C3AED]/35",
  isa: "bg-[#EA580C]/15 text-[#FDBA74] ring-[#EA580C]/35",
  pension: "bg-[#DB2777]/15 text-[#FDA4AF] ring-[#DB2777]/35"
};

function accountTypeBadgeClass(type: AccountType) {
  return `ring ring-inset ${ACCOUNT_TYPE_BADGE_CLASS[type] ?? "bg-elevated text-default ring-accented"}`;
}

const ACCOUNT_TYPE_LINE_COLOR: Record<AccountType, string> = {
  current: "#93C5FD",
  savings: "#4ADE80",
  stocks: "#C4B5FD",
  isa: "#FDBA74",
  pension: "#FDA4AF"
};

function accountTypeLineColor(type: AccountType) {
  return ACCOUNT_TYPE_LINE_COLOR[type] ?? "#94A3B8";
}

const gbp = new Intl.NumberFormat("en-GB", {
  style: "currency",
  currency: "GBP"
});

function formatGBP(value: number) {
  return gbp.format(value);
}

interface BalanceHistory {
  startDay: number
  balances: number[]
}

interface ActivityData {
  values: Array<number | null>
  delta: number
}

const MS_PER_DAY = 86_400_000;

function getTodayDayNumber() {
  const now = new Date();
  return Math.floor(Date.UTC(now.getFullYear(), now.getMonth(), now.getDate()) / MS_PER_DAY);
}

const todayDay = getTodayDayNumber();

function dayNumberToIso(dayNumber: number) {
  return new Date(dayNumber * MS_PER_DAY).toISOString().slice(0, 10);
}

function formatShortDayNumber(dayNumber?: number) {
  if (typeof dayNumber !== "number" || !Number.isFinite(dayNumber)) {
    return "";
  }

  return formatShortDate(dayNumberToIso(dayNumber));
}

function hashStringToSeed(str: string) {
  // FNV-1a 32-bit
  let hash = 2_166_136_261;
  for (let i = 0; i < str.length; i++) {
    hash ^= str.charCodeAt(i);
    hash = Math.imul(hash, 16_777_619);
  }
  return hash >>> 0;
}

function mulberry32(seed: number) {
  let a = seed;
  return () => {
    a |= 0;
    a = a + 0x6D2B79F5 | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = t + Math.imul(t ^ (t >>> 7), 61 | t) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function randInt(rng: () => number, min: number, max: number) {
  return Math.floor(rng() * (max - min + 1)) + min;
}

function round2(n: number) {
  return Math.round(n * 100) / 100;
}

const ACTIVITY_PERIOD_POINTS: Record<ActivityPeriod, number> = {
  "1W": 7,
  "1M": 30,
  "3M": 90,
  "6M": 180
};

const HISTORY_RANGE_BY_TYPE: Record<AccountType, { min: number, max: number }> = {
  current: { min: 7, max: 120 },
  savings: { min: 14, max: 220 },
  stocks: { min: 14, max: 220 },
  isa: { min: 30, max: 220 },
  pension: { min: 90, max: 220 }
};

const VOLATILITY_BY_TYPE: Record<AccountType, number> = {
  current: 0.02,
  savings: 0.006,
  stocks: 0.03,
  isa: 0.02,
  pension: 0.01
};

const accountHistoryById = computed<Record<string, BalanceHistory>>(() => {
  const out: Record<string, BalanceHistory> = {};

  for (const account of rawAccounts.value) {
    const seed = hashStringToSeed(account.id);
    const rng = mulberry32(seed);

    const { min, max } = HISTORY_RANGE_BY_TYPE[account.type] ?? { min: 14, max: 180 };
    const daysAgo = randInt(rng, min, max);

    const startDay = todayDay - daysAgo;
    const points = daysAgo + 1;

    const balances = Array.from({ length: points }, () => 0);
    const scale = Math.max(Math.abs(account.currentBalance), 1000);
    const volatility = VOLATILITY_BY_TYPE[account.type] ?? 0.01;

    balances[points - 1] = round2(account.currentBalance);

    for (let i = points - 2; i >= 0; i--) {
      const noise = (rng() - 0.5) * 2;
      const delta = noise * volatility * scale;
      const next = balances[i + 1] ?? 0;
      balances[i] = round2(Math.max(0, next - delta));
    }

    out[account.id] = { startDay, balances };
  }

  return out;
});

function getGroupedFirstChange(row: TableRow<Account>) {
  return Number(row.getValue("firstChange") ?? Number.NaN);
}

function valuesForPeriod(history: BalanceHistory, periodStartDay: number, points: number) {
  const startIndex = periodStartDay - history.startDay;
  const missing = startIndex < 0 ? -startIndex : 0;
  const sliceStart = startIndex < 0 ? 0 : startIndex;
  const sliceLen = Math.max(0, points - missing);

  const values: Array<number | null> = [];
  for (let i = 0; i < missing; i++) {
    values.push(null);
  }
  for (let i = 0; i < sliceLen; i++) {
    values.push(history.balances[sliceStart + i] ?? null);
  }

  return values;
}

function deltaFromValues(values: Array<number | null>) {
  const first = values.find((v) => typeof v === "number");
  const last = values.at(-1);

  if (typeof first !== "number" || typeof last !== "number") {
    return 0;
  }

  return last - first;
}

const activityByAccountId = computed<Record<string, ActivityData>>(() => {
  const points = ACTIVITY_PERIOD_POINTS[activityPeriod.value];
  const periodStartDay = todayDay - (points - 1);
  const histories = accountHistoryById.value;

  const out: Record<string, ActivityData> = {};
  for (const account of rawAccounts.value) {
    const history = histories[account.id];
    if (!history) {
      continue;
    }

    const values = valuesForPeriod(history, periodStartDay, points);
    out[account.id] = {
      values,
      delta: deltaFromValues(values)
    };
  }

  return out;
});

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

    out[i] = hasValue ? round2(sum) : null;
  }

  return out;
}

function getRowActivityValues(row: TableRow<Account>) {
  if (!row.getIsGrouped()) {
    return activityByAccountId.value[row.original.id]?.values ?? [];
  }

  const accounts = leafAccountsFromRow(row);
  const seriesList = accounts
    .map((a) => activityByAccountId.value[a.id]?.values)
    .filter(Boolean) as Array<Array<number | null>>;

  return aggregateSeries(seriesList);
}

function getRowActivityColor(row: TableRow<Account>) {
  if (!row.getIsGrouped()) {
    return accountTypeLineColor(row.original.type);
  }

  const groupingId = row.groupingColumnId as string | undefined;
  if (groupingId === "type_group") {
    return accountTypeLineColor(row.getValue("type_group") as AccountType);
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
    return accountTypeLabel(row.getValue("type_group") as AccountType);
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
    accessorKey: "institution",
    enableSorting: false
  },
  {
    id: "type_group",
    accessorKey: "type",
    enableSorting: false
  },
  {
    accessorKey: "name",
    header: ({ column }) => sortableHeader(column, "Name")
  },
  {
    accessorKey: "institution",
    header: ({ column }) => sortableHeader(column, "Institution")
  },
  {
    accessorKey: "type",
    header: ({ column }) => sortableHeader(column, "Type")
  },
  {
    id: "firstChange",
    header: ({ column }) => sortableHeader(column, "First change"),
    accessorFn: (row) => accountHistoryById.value[row.id]?.startDay ?? todayDay,
    aggregationFn: (_columnId, leafRows: any[]) => {
      const days = leafRows
        .map((r) => accountHistoryById.value[r?.original?.id]?.startDay)
        .filter((d): d is number => typeof d === "number");

      return days.length ? Math.min(...days) : todayDay;
    }
  },
  {
    id: "lastChange",
    accessorKey: "lastBalanceChangeDate",
    header: ({ column }) => sortableHeader(column, "Last change"),
    aggregationFn: "max"
  },
  {
    id: "activity",
    header: ({ column }) => sortableHeader(column, `Activity (${activityPeriod.value})`),
    accessorFn: (row) => activityByAccountId.value[row.id]?.delta ?? 0,
    aggregationFn: (_columnId, leafRows: any[]) => {
      const seriesList = leafRows
        .map((r) => activityByAccountId.value[r?.original?.id]?.values)
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
    accessorKey: "currentBalance",
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
