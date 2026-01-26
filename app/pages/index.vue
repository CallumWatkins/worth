<template>
  <UContainer>
    <UPageHeader
      title="Balance Overview"
      description="A summary of your balances across all accounts"
      :links="links"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />
    <UPageBody class="space-y-8">
      <UPageCard
        title="Total Current Balance"
        orientation="horizontal"
        variant="subtle"
        :ui="{
          container: 'lg:grid-cols-[1fr_auto]',
          title: 'uppercase text-muted text-xs'
        }"
      >
        <template #description>
          <div>
            <span class="text-[2.5rem] text-4xl font-bold text-default mr-4">£245,890.12</span>
            <span class="inline-flex gap-1 leading-none">
              <UIcon name="i-lucide-arrow-up" class="size-4 text-success" />
              <span class="text-success">2.4%</span>
              <span class="text-muted text-xs self">vs last month</span>
            </span>
          </div>
        </template>
        <div class="flex gap-4">
          <UPageCard
            title="Monthly Yield"
            description="+£1,240.82"
            variant="subtle"
            :ui="{
              title: 'text-muted text-xs whitespace-nowrap',
              description: 'text-xl font-bold text-success whitespace-nowrap'
            }"
          />
          <UPageCard
            title="Active Accounts"
            description="12"
            variant="subtle"
            :ui="{
              title: 'text-muted text-xs whitespace-nowrap',
              description: 'text-xl font-bold text-default whitespace-nowrap'
            }"
          />
        </div>
      </UPageCard>
      <UPageGrid>
        <UPageCard
          title="Balance by Account Type"
          description="Portfolio allocation"
          :ui="{
            body: 'w-full'
          }"
        >
          <VChart
            :option="balanceAllocationOption"
            autoresize
            style="height: 300px; width: 100%"
            @legendselectchanged="onAllocationLegendSelectChanged"
          />
        </UPageCard>
        <UPageCard
          class="col-span-2"
          :ui="{
            body: 'w-full'
          }"
        >
          <template #body>
            <div class="flex flex-row items-center justify-between">
              <div>
                <div class="text-base text-pretty font-semibold text-highlighted">
                  Total Balance Over Time
                </div>
                <div class="text-[15px] text-pretty text-muted mt-1">
                  Growth trajectory over the last 6 months
                </div>
              </div>
              <div class="mt-4 sm:mt-0 sm:ml-6 shrink-0">
                <UTabs
                  :items="[{ label: '1M' }, { label: '6M' }, { label: '1Y' }]" color="neutral"
                  default-value="1"
                  size="sm"
                  :ui="{
                    indicator: 'bg-neutral-700',
                    label: 'text-neutral-300'
                  }"
                />
              </div>
            </div>
          </template>
          <VChart
            :option="balanceOverTimeOption"
            autoresize
            style="height: 300px; width: 100%"
          />
        </UPageCard>
      </UPageGrid>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { ButtonProps } from "@nuxt/ui";

const links = ref<ButtonProps[]>([
  {
    label: "Export Data",
    icon: "i-lucide-download",
    to: "/data/export",
    variant: "subtle"
  },
  {
    label: "Add New Account",
    icon: "i-lucide-plus",
    to: "/accounts/new",
    color: "primary",
    variant: "solid"
  }
]);

const formatIsoDate = (d: Date) => d.toISOString().slice(0, 10);
const monthShort = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;

const formatShortGBP = (value: number) => {
  const abs = Math.abs(value);
  const sign = value < 0 ? "-" : "";
  if (abs >= 1_000_000_000) return `${sign}£${Math.round(abs / 1_000_000_000)}b`;
  if (abs >= 1_000_000) return `${sign}£${Math.round(abs / 1_000_000)}m`;
  if (abs >= 1_000) return `${sign}£${Math.round(abs / 1_000)}k`;
  return `${sign}£${Math.round(abs)}`;
};

// Daily data for ~6 months (deterministic placeholder)
const days = 183;
const today = new Date();
const dailyDates: string[] = [];
const dailyValues: number[] = [];

for (let i = days - 1; i >= 0; i--) {
  const d = new Date(today);
  d.setDate(d.getDate() - i);
  dailyDates.push(formatIsoDate(d));

  // Smooth-ish upward trend + gentle seasonality (no randomness)
  const t = (days - 1 - i);
  const value = 245_000 + t * 35 + Math.sin(t / 9) * 900 + Math.cos(t / 21) * 500;
  dailyValues.push(Math.round(value * 100) / 100);
}

const balanceOverTimeOption: ECOption = {
  backgroundColor: "transparent",
  tooltip: {
    trigger: "axis",
    axisPointer: { type: "line" }
  },
  grid: {
    left: 0,
    right: 0,
    top: 0,
    bottom: 0
  },
  xAxis: {
    type: "category",
    data: dailyDates,
    boundaryGap: false,
    axisTick: { show: false },
    axisLine: { show: false },
    splitLine: { show: false },
    axisLabel: {
      hideOverlap: true,
      // show mid-month (e.g. 2026-02-15) as "Feb"
      interval: (_index: number, value: string) => value.endsWith("-15"),
      formatter: (value: string) => {
        const m = Number(value.split("-")[1] ?? 0);
        return monthShort[m - 1] ?? "";
      }
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
      data: dailyValues,
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

interface AllocationDatum {
  name: "Current" | "Savings" | "Stocks"
  value: number
  color: string
  glow: string
  glowEmphasis: string
}

const allocationData: AllocationDatum[] = [
  { name: "Current", value: 72_400, color: "#3b82f6", glow: "rgba(59, 130, 246, 0.55)", glowEmphasis: "rgba(59, 130, 246, 0.85)" },
  { name: "Savings", value: 128_900, color: "#22c55e", glow: "rgba(34, 197, 94, 0.55)", glowEmphasis: "rgba(34, 197, 94, 0.85)" },
  { name: "Stocks", value: 44_590, color: "#a855f7", glow: "rgba(168, 85, 247, 0.55)", glowEmphasis: "rgba(168, 85, 247, 0.85)" }
];

const allocationSelected = ref<Record<AllocationDatum["name"], boolean>>({
  Current: true,
  Savings: true,
  Stocks: true
});

const buildBalanceAllocationOption = (selected: Record<string, boolean>): ECOption => {
  const totalVisible = allocationData.reduce((sum, d) => sum + (selected[d.name] === false ? 0 : d.value), 0);
  const percentByName = Object.fromEntries(
    allocationData.map((d) => {
      const value = selected[d.name] === false ? 0 : d.value;
      const pct = totalVisible > 0 ? Math.round((value / totalVisible) * 100) : 0;
      return [d.name, pct];
    })
  ) as Record<string, number>;

  return {
    backgroundColor: "transparent",
    tooltip: {
      trigger: "item",
      valueFormatter: (value: unknown) => {
        const n = typeof value === "number" ? value : Number(value);
        if (Number.isFinite(n)) return `£${n.toLocaleString("en-GB", { maximumFractionDigits: 0 })}`;
        return String(value);
      }
    },
    legend: {
      bottom: 0,
      left: "center",
      itemWidth: 10,
      itemHeight: 10,
      icon: "circle",
      selected,
      textStyle: {
        color: "#a3a3a3"
      },
      formatter: (name: string) => `${name}  ${percentByName[name] ?? 0}%`
    },
    graphic: [
      {
        type: "text",
        left: "center",
        top: "39%",
        style: {
          text: formatShortGBP(totalVisible),
          fontSize: 28,
          fontWeight: 700,
          fill: "#e5e5e5",
          align: "center",
          verticalAlign: "middle"
        }
      }
    ],
    series: [
      {
        name: "Allocation",
        type: "pie",
        radius: ["62%", "72%"],
        center: ["50%", "42%"],
        padAngle: 2,
        avoidLabelOverlap: true,
        label: { show: false },
        labelLine: { show: false },
        data: allocationData.map((d) => ({
          name: d.name,
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

const balanceAllocationOption = ref<ECOption>(buildBalanceAllocationOption(allocationSelected.value));

const onAllocationLegendSelectChanged = (params: { selected?: Record<string, boolean> }) => {
  const selected = params.selected ?? {};
  allocationSelected.value = {
    Current: selected.Current !== false,
    Savings: selected.Savings !== false,
    Stocks: selected.Stocks !== false
  };
  balanceAllocationOption.value = buildBalanceAllocationOption(allocationSelected.value);
};
</script>
