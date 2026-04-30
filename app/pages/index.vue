<template>
  <UContainer>
    <UAlert
      v-if="dashboardQuery.isError"
      class="pt-6"
      color="error"
      variant="subtle"
      :title="dashboardQuery.error.message"
    />

    <EmptyAppOnboarding v-else-if="dashboardQuery.isSuccess && dashboardQuery.data.total_accounts === 0" />

    <template v-else-if="dashboardQuery.isSuccess">
      <UPageHeader
        title="Balance Overview"
        description="A summary of your balances across all accounts"
        :ui="{
          root: 'pb-0 border-none',
          description: 'mt-1'
        }"
      />
      <UPageBody class="space-y-8">
        <UPageCard
          title="Total Current Balance"
          orientation="horizontal"
          variant="outline"
          spotlight
          :ui="{
            container: 'lg:grid-cols-[1fr_auto]',
            title: 'text-muted text-sm',
            spotlight: 'bg-default/95'
          }"
        >
          <template #description>
            <div>
              <NumberFlow
                :key="`total:${dashboardNumberAnimationKey}`"
                class="dashboard-total-balance mr-4 text-[2.5rem] text-4xl font-bold text-default"
                :value="animatedTotalBalance"
                :locales="appLocaleCode"
                :format="{ style: 'currency', currency: 'GBP' }"
                :plugins="numberFlowPlugins"
                :transform-timing="{ duration: 0 }"
                :spin-timing="numberFlowTiming"
                :opacity-timing="numberFlowTiming"
              />
              <span class="inline-flex items-baseline gap-1 leading-none">
                <NumberFlow
                  v-if="targetChangePct != null"
                  :key="`change:${dashboardNumberAnimationKey}`"
                  class="dashboard-change-pct"
                  :class="changeClass"
                  :value="animatedChangePct"
                  :locales="appLocaleCode"
                  :format="{ signDisplay: 'always', minimumFractionDigits: 1, maximumFractionDigits: 1 }"
                  :plugins="numberFlowPlugins"
                  :transform-timing="{ duration: 0 }"
                  :spin-timing="numberFlowTiming"
                  :opacity-timing="numberFlowTiming"
                  suffix="%"
                />
                <span v-else :class="changeClass">—</span>
                <span class="text-muted text-xs">vs last month</span>
              </span>
            </div>
          </template>
          <div class="flex gap-4 *:min-w-32">
            <UPageCard
              title="Monthly Yield"
              variant="subtle"
              :ui="{
                title: 'text-muted text-sm whitespace-nowrap'
              }"
            >
              <template #description>
                <NumberFlow
                  v-if="targetMonthlyYield != null"
                  :key="`yield:${dashboardNumberAnimationKey}`"
                  class="dashboard-monthly-yield"
                  :class="monthlyYieldDescriptionClass"
                  :value="animatedMonthlyYield"
                  :locales="appLocaleCode"
                  :format="{ style: 'currency', currency: 'GBP', signDisplay: 'always' }"
                  :plugins="numberFlowPlugins"
                  :transform-timing="{ duration: 0 }"
                  :spin-timing="numberFlowTiming"
                  :opacity-timing="numberFlowTiming"
                />
                <span v-else :class="monthlyYieldDescriptionClass">—</span>
              </template>
            </UPageCard>
            <UPageCard
              :to="{ name: 'accounts' }"
              title="Active Accounts"
              variant="subtle"
              :ui="{
                title: 'text-muted text-sm whitespace-nowrap'
              }"
            >
              <template #description>
                <NumberFlow
                  :key="`accounts:${dashboardNumberAnimationKey}`"
                  class="dashboard-card-count text-xl font-bold text-default whitespace-nowrap"
                  :value="animatedActiveAccounts"
                  :locales="appLocaleCode"
                  :format="{ maximumFractionDigits: 0 }"
                  :plugins="numberFlowPlugins"
                  :transform-timing="{ duration: 0 }"
                  :spin-timing="numberFlowTiming"
                  :opacity-timing="numberFlowTiming"
                />
              </template>
            </UPageCard>
            <UPageCard
              :to="{ name: 'institutions' }"
              title="Active Institutions"
              variant="subtle"
              :ui="{
                title: 'text-muted text-sm whitespace-nowrap'
              }"
            >
              <template #description>
                <NumberFlow
                  :key="`institutions:${dashboardNumberAnimationKey}`"
                  class="dashboard-card-count text-xl font-bold text-default whitespace-nowrap"
                  :value="animatedActiveInstitutions"
                  :locales="appLocaleCode"
                  :format="{ maximumFractionDigits: 0 }"
                  :plugins="numberFlowPlugins"
                  :transform-timing="{ duration: 0 }"
                  :spin-timing="numberFlowTiming"
                  :opacity-timing="numberFlowTiming"
                />
              </template>
            </UPageCard>
          </div>
        </UPageCard>
        <UPageGrid>
          <UPageCard
            title="Balance by Account Type"
            description="Portfolio allocation"
            spotlight
            :ui="{
              container: 'grid min-w-0 gap-y-2',
              body: 'w-full',
              spotlight: 'bg-default/95'
            }"
          >
            <div class="relative left-1/2 w-[calc(100%+24px)] min-w-0 -translate-x-1/2">
              <VChart
                ref="allocationChart"
                :option="balanceAllocationOption"
                autoresize
                style="height: 300px; width: 100%"
                @mouseover="onAllocationChartHover($event, true)"
                @mouseout="onAllocationChartHover($event, false)"
              />
              <NumberFlow
                :key="`allocation:${dashboardNumberAnimationKey}`"
                class="dashboard-allocation-total pointer-events-none absolute left-1/2 top-1/2 z-10 -translate-x-1/2 -translate-y-1/2 text-[28px] font-bold text-neutral-200"
                :value="animatedAllocationTotal"
                :locales="appLocaleCode"
                :format="{
                  style: 'currency',
                  currency: 'GBP',
                  notation: 'compact',
                  compactDisplay: 'short',
                  minimumFractionDigits: Math.abs(allocationVisibleTotal) < 1000 ? 2 : 0,
                  maximumFractionDigits: Math.abs(allocationVisibleTotal) < 1000 ? 2 : 1
                }"
                :plugins="numberFlowPlugins"
                :transform-timing="allocationTotalUsesDashboardTiming ? { duration: 0 } : allocationToggleNumberFlowTiming"
                :spin-timing="allocationTotalUsesDashboardTiming ? numberFlowTiming : allocationToggleNumberFlowTiming"
                :opacity-timing="allocationTotalUsesDashboardTiming ? numberFlowTiming : allocationToggleNumberFlowTiming"
              />
            </div>
            <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-2 pt-1">
              <button
                v-for="item in allocationData"
                :key="item.label"
                type="button"
                class="inline-flex cursor-pointer items-center gap-2 text-[13px] leading-none text-neutral-400 transition hover:text-neutral-200 focus-visible:outline-none focus-visible:text-neutral-200"
                :class="isAllocationSelected(item.label) || isAllocationLegendActive(item.label) ? 'opacity-100' : 'opacity-45'"
                :aria-pressed="isAllocationSelected(item.label)"
                @mouseenter="onAllocationLegendHover(item.label, true)"
                @mouseleave="onAllocationLegendHover(item.label, false)"
                @focus="onAllocationLegendHover(item.label, true)"
                @blur="onAllocationLegendHover(item.label, false)"
                @click="toggleAllocationLegendItem(item.label)"
              >
                <span
                  class="size-2.5 shrink-0 rounded-full transition duration-200 ease-out"
                  :style="{
                    backgroundColor: item.color,
                    boxShadow: isAllocationHighlighted(item.label) ? `0 0 2px ${item.glowEmphasis}, 0 0 8px ${item.glow}` : `0 0 4px ${item.glow}`,
                    transform: isAllocationHighlighted(item.label) ? 'scale(1.14)' : 'scale(1)'
                  }"
                />
                <span class="leading-none">{{ item.label }}</span>
              </button>
            </div>
          </UPageCard>
          <UPageCard
            class="col-span-2"
            spotlight
            :ui="{
              body: 'w-full',
              spotlight: 'bg-default/95'
            }"
          >
            <template #body>
              <div class="flex flex-row items-center justify-between">
                <div>
                  <div class="text-base text-pretty font-semibold text-highlighted">
                    Total Balance Over Time
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
                <div class="ml-6 shrink-0">
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
        </UPageGrid>
      </UPageBody>
    </template>
  </UContainer>
</template>

<script lang="ts" setup>
import type { BalanceOverTimePeriod } from "~/generated/bindings";
import NumberFlow, { continuous } from "@number-flow/vue";
import { useQuery } from "@tanstack/vue-query";
import { useLocaleFormatters } from "~/composables/useLocaleFormatters";

const { formatCurrency, formatDate } = useLocaleFormatters();
const { code: appLocaleCode } = useAppLocale();

const darkTooltipBase = {
  backgroundColor: "rgba(10, 10, 10, 0.95)",
  borderColor: "rgba(82, 82, 91, 0.6)",
  borderWidth: 1,
  textStyle: { color: "#f5f5f5" },
  extraCssText: "border-radius: 10px; box-shadow: 0 10px 30px rgba(0,0,0,0.45); padding: 10px 12px;"
} as const;

const api = useApi();

const balanceOverTimePeriod = ref<BalanceOverTimePeriod>("6M");

const dashboardQuery = proxyRefs(useQuery({
  queryKey: queryKeys.dashboard.summary(),
  queryFn: api.dashboardGet
}));

const balanceOverTimeQuery = proxyRefs(useQuery({
  queryKey: computed(() => queryKeys.dashboard.balanceOverTime(balanceOverTimePeriod.value)),
  queryFn: async () => api.dashboardBalanceOverTime(balanceOverTimePeriod.value)
}));

const animatedTotalBalance = ref(0);
const animatedChangePct = ref(0);
const animatedMonthlyYield = ref(0);
const animatedActiveAccounts = ref(0);
const animatedActiveInstitutions = ref(0);
const animatedAllocationTotal = ref(0);
const allocationTotalUsesDashboardTiming = ref(true);
const dashboardNumberAnimationKey = ref(0);
const numberFlowPlugins = [continuous];
const numberFlowTiming = {
  duration: 1000,
  easing: "cubic-bezier(0,0,0.35,1)"
} satisfies EffectTiming;
const allocationToggleNumberFlowTiming = {
  duration: 500,
  easing: "cubic-bezier(0.65,0,0.35,1)"
} satisfies EffectTiming;

const targetChangePct = computed(() => {
  const changePct = dashboardQuery.data?.change_vs_last_month_pct;
  return changePct == null ? null : changePct;
});
const targetMonthlyYield = computed(() => {
  const monthlyYieldMinor = dashboardQuery.data?.monthly_yield_minor;
  return monthlyYieldMinor == null ? null : convertCurrencyMinorUnitsToMajorAmount(monthlyYieldMinor);
});

const balanceOverTimePeriodItems = computed(() => {
  const disabled = balanceOverTimeQuery.isFetching;
  return [
    { label: "1M", value: "1M", disabled },
    { label: "6M", value: "6M", disabled },
    { label: "1Y", value: "1Y", disabled },
    { label: "MAX", value: "MAX", disabled }
  ];
});

const changeClass = computed(() => {
  if (dashboardQuery.data?.change_vs_last_month_pct == null) return "text-muted";
  return (dashboardQuery.data?.change_vs_last_month_pct ?? 0) >= 0 ? "text-success" : "text-error";
});

const monthlyYieldDescriptionClass = computed(() => {
  if (dashboardQuery.data?.monthly_yield_minor == null) return "text-xl font-bold text-muted whitespace-nowrap";
  return `text-xl font-bold ${dashboardQuery.data?.monthly_yield_minor >= 0 ? "text-success" : "text-error"} whitespace-nowrap`;
});

const balanceOverTimeOption = computed<ECOption>(() => {
  const points = balanceOverTimeQuery.data ?? [];
  const dates = points.map((p) => p.date);
  const values = points.map((p) => convertCurrencyMinorUnitsToMajorAmount(p.balance_minor));

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
      }
    },
    grid: {
      left: 0,
      right: 0,
      top: 0,
      bottom: 0
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
      show: false,
      scale: true,
      splitLine: { show: false }
    },
    series: [
      {
        name: "Balance",
        type: "line",
        silent: true,
        data: values,
        smooth: true,
        showSymbol: false,
        lineStyle: {
          width: 5,
          color: "#22c55e",
          shadowBlur: 10,
          shadowColor: "rgba(34, 197, 94, 0.6)",
          shadowOffsetX: 0,
          shadowOffsetY: 0
        },
        itemStyle: { color: "#22c55e" },
        areaStyle: {
          color: {
            type: "linear",
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: "rgba(34, 197, 94, 0.35)" },
              { offset: 1, color: "rgba(34, 197, 94, 0)" }
            ]
          }
        },
        emphasis: {
          focus: "series",
          lineStyle: {
            shadowBlur: 14,
            shadowColor: "rgba(34, 197, 94, 0.85)"
          }
        }
      }
    ]
  };
});

interface AllocationDatum {
  label: string
  value: number
  color: string
  glow: string
  glowEmphasis: string
}

const allocationData = computed<AllocationDatum[]>(() => {
  const rows = dashboardQuery.data?.allocation_by_type ?? [];
  return rows.map((r) => {
    const meta = ACCOUNT_TYPE_META[r.account_type];
    return {
      label: meta.label,
      value: convertCurrencyMinorUnitsToMajorAmount(r.balance_minor),
      color: meta.color,
      glow: meta.glow,
      glowEmphasis: meta.glowEmphasis
    };
  });
});

const allocationSelected = ref<Record<string, boolean>>({});
const allocationChart = ref<{ dispatchAction: (payload: { type: "highlight" | "downplay", seriesIndex: number, name: string }) => void } | null>(null);
const activeAllocationLegendLabel = ref<string | null>(null);
const highlightedAllocationLabel = ref<string | null>(null);

interface AllocationChartEvent {
  componentType?: string
  seriesType?: string
  name?: string
}

watchEffect(() => {
  for (const d of allocationData.value) {
    if (allocationSelected.value[d.label] === undefined) {
      allocationSelected.value[d.label] = true;
    }
  }
});

const allocationVisibleTotal = computed(() => allocationData.value.reduce((sum, d) => sum + (allocationSelected.value[d.label] === false ? 0 : d.value), 0));

let dashboardNumberAnimationRun = 0;

const replayDashboardNumberAnimations = async () => {
  if (dashboardQuery.data == null) return;

  const run = ++dashboardNumberAnimationRun;
  dashboardNumberAnimationKey.value += 1;
  allocationTotalUsesDashboardTiming.value = true;
  animatedTotalBalance.value = 0;
  animatedChangePct.value = 0;
  animatedMonthlyYield.value = 0;
  animatedActiveAccounts.value = 0;
  animatedActiveInstitutions.value = 0;
  animatedAllocationTotal.value = 0;

  await nextTick();
  for (let i = 0; i < 2; i++) {
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve());
    });
  }

  if (run !== dashboardNumberAnimationRun) return;

  animatedTotalBalance.value = convertCurrencyMinorUnitsToMajorAmount(dashboardQuery.data?.total_balance_minor ?? 0);
  animatedChangePct.value = targetChangePct.value ?? 0;
  animatedMonthlyYield.value = targetMonthlyYield.value ?? 0;
  animatedActiveAccounts.value = dashboardQuery.data?.active_accounts ?? 0;
  animatedActiveInstitutions.value = dashboardQuery.data?.active_institutions ?? 0;
  animatedAllocationTotal.value = allocationVisibleTotal.value;
};

watch(
  () => dashboardQuery.data,
  () => void replayDashboardNumberAnimations(),
  { flush: "post" }
);

onMounted(() => void replayDashboardNumberAnimations());

const buildBalanceAllocationOption = (selected: Record<string, boolean>, data: AllocationDatum[]): ECOption => {
  const visibleData = data.filter((d) => selected[d.label] !== false);

  return {
    backgroundColor: "transparent",
    tooltip: {
      ...darkTooltipBase,
      trigger: "item",
      valueFormatter: (value: unknown) => {
        const n = typeof value === "number" ? value : Number(value);
        if (Number.isFinite(n)) return formatCurrency(n, "GBP", { maximumFractionDigits: 0 });
        return String(value);
      }
    },
    series: [
      {
        name: "Allocation",
        type: "pie",
        cursor: "default",
        radius: ["66%", "76%"],
        center: ["50%", "50%"],
        padAngle: 2,
        avoidLabelOverlap: true,
        label: { show: false },
        labelLine: { show: false },
        data: visibleData.map((d) => ({
          name: d.label,
          cursor: "default",
          value: d.value,
          itemStyle: {
            color: d.color,
            borderRadius: 8,
            shadowBlur: 12,
            shadowColor: d.glow
          },
          emphasis: {
            itemStyle: {
              shadowBlur: 18,
              shadowColor: d.glowEmphasis
            }
          }
        }))
      }
    ]
  };
};

const balanceAllocationOption = computed<ECOption>(() => {
  return buildBalanceAllocationOption(allocationSelected.value, allocationData.value);
});

const isAllocationSelected = (label: string) => allocationSelected.value[label] !== false;

const isAllocationLegendActive = (label: string) => activeAllocationLegendLabel.value === label;

const isAllocationHighlighted = (label: string) => highlightedAllocationLabel.value === label && isAllocationSelected(label);

const dispatchAllocationAction = (type: "highlight" | "downplay", label: string) => {
  allocationChart.value?.dispatchAction({ type, seriesIndex: 0, name: label });
};

const onAllocationLegendHover = (label: string, isHighlighted: boolean) => {
  if (isHighlighted) {
    activeAllocationLegendLabel.value = label;
    if (isAllocationSelected(label)) {
      highlightedAllocationLabel.value = label;
      dispatchAllocationAction("highlight", label);
    }
    return;
  }

  if (activeAllocationLegendLabel.value === label) {
    activeAllocationLegendLabel.value = null;
  }
  if (highlightedAllocationLabel.value === label) {
    highlightedAllocationLabel.value = null;
  }
  dispatchAllocationAction("downplay", label);
};

const onAllocationChartHover = (params: AllocationChartEvent, isHighlighted: boolean) => {
  if (params.componentType !== "series" || params.seriesType !== "pie" || params.name == null || params.name === "") return;

  if (isHighlighted) {
    highlightedAllocationLabel.value = params.name;
  } else if (highlightedAllocationLabel.value === params.name) {
    highlightedAllocationLabel.value = null;
  }
};

const toggleAllocationLegendItem = (label: string) => {
  const isEnabling = !isAllocationSelected(label);

  allocationSelected.value = {
    ...allocationSelected.value,
    [label]: isEnabling
  };
  allocationTotalUsesDashboardTiming.value = false;
  animatedAllocationTotal.value = allocationVisibleTotal.value;

  if (isEnabling && activeAllocationLegendLabel.value === label) {
    highlightedAllocationLabel.value = label;
    void nextTick(() => {
      dispatchAllocationAction("highlight", label);
    });
    return;
  }

  if (!isEnabling && highlightedAllocationLabel.value === label) {
    highlightedAllocationLabel.value = null;
  }
  dispatchAllocationAction("downplay", label);
};
</script>

<style scoped>
.dashboard-total-balance,
.dashboard-change-pct,
.dashboard-monthly-yield,
.dashboard-card-count,
.dashboard-allocation-total {
  display: inline-block;
  font-variant-numeric: tabular-nums;
}

.dashboard-total-balance {
  line-height: 0.9;
  --number-flow-mask-height: 0.18em;
  --number-flow-mask-width: 0.25em;
}

.dashboard-change-pct,
.dashboard-monthly-yield,
.dashboard-card-count,
.dashboard-allocation-total {
  line-height: 1;
  --number-flow-mask-height: 0.15em;
  --number-flow-mask-width: 0.2em;
}
</style>
