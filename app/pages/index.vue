<template>
  <UContainer>
    <UPageHeader
      title="Balance Overview"
      description="A summary of your balances across all accounts"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />
    <UPageBody class="space-y-8">
      <UAlert
        v-if="dashboardQuery.isError"
        color="error"
        variant="subtle"
        :title="dashboardQuery.error!.message ?? 'Failed to load dashboard'"
      />
      <UPageCard
        title="Total Current Balance"
        orientation="horizontal"
        variant="outline"
        spotlight
        :ui="{
          container: 'lg:grid-cols-[1fr_auto]',
          title: 'uppercase text-muted text-xs',
          spotlight: 'bg-default/95'
        }"
      >
        <template #description>
          <div>
            <span class="text-[2.5rem] text-4xl font-bold text-default mr-4">{{ totalBalanceLabel }}</span>
            <span class="inline-flex gap-1 leading-none">
              <UIcon :name="changeIcon" class="size-4" :class="[changeClass]" />
              <span :class="changeClass">{{ changePctLabel }}</span>
              <span class="text-muted text-xs self">vs last month</span>
            </span>
          </div>
        </template>
        <div class="flex gap-4">
          <UPageCard
            title="Monthly Yield"
            :description="monthlyYieldLabel"
            variant="subtle"
            :ui="{
              title: 'text-muted text-xs whitespace-nowrap',
              description: monthlyYieldDescriptionClass
            }"
          />
          <UPageCard
            to="/accounts"
            title="Active Accounts"
            :description="activeAccountsLabel"
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
          spotlight
          :ui="{
            body: 'w-full',
            spotlight: 'bg-default/95'
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

      <UPageCard
        title="Hello world"
        description="Tauri command test"
        variant="outline"
      >
        <div class="flex flex-col gap-3">
          <div class="flex flex-col sm:flex-row gap-3 sm:items-end">
            <UFormField label="Name" name="helloName" class="flex-1">
              <UInput v-model="helloName" />
            </UFormField>

            <UButton
              label="Say hello"
              color="primary"
              variant="solid"
              :loading="helloMutation.isPending"
              @click="sayHello"
            />
          </div>

          <div v-if="helloMutation.data" class="font-medium text-highlighted">
            {{ helloMutation.data }}
          </div>

          <UAlert
            v-else-if="helloMutation.isError"
            color="error"
            variant="subtle"
            :title="helloMutation.error!.message ?? 'Something went wrong'"
          />
        </div>
      </UPageCard>
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { AccountTypeName } from "~/bindings";

import { useMutation, useQuery } from "@tanstack/vue-query";
import { computed, proxyRefs, watchEffect } from "vue";

import { ACCOUNT_TYPE_META, accountTypeMetaLoose } from "~/utils/account-type-meta";

const monthShort = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;

const gbp = new Intl.NumberFormat("en-GB", {
  style: "currency",
  currency: "GBP"
});

const formatGBPMinor = (minor: number) => {
  return gbp.format(minor / 100);
};

const formatShortGBP = (value: number) => {
  const abs = Math.abs(value);
  const sign = value < 0 ? "-" : "";
  if (abs >= 1_000_000_000) return `${sign}£${Math.round(abs / 1_000_000_000)}b`;
  if (abs >= 1_000_000) return `${sign}£${Math.round(abs / 1_000_000)}m`;
  if (abs >= 1_000) return `${sign}£${Math.round(abs / 1_000)}k`;
  return `${sign}£${Math.round(abs)}`;
};

const darkTooltipBase = {
  backgroundColor: "rgba(10, 10, 10, 0.95)",
  borderColor: "rgba(82, 82, 91, 0.6)",
  borderWidth: 1,
  textStyle: { color: "#f5f5f5" },
  extraCssText: "border-radius: 10px; box-shadow: 0 10px 30px rgba(0,0,0,0.45); padding: 10px 12px;"
} as const;

const api = useApi();

const dashboardQuery = proxyRefs(useQuery({
  queryKey: ["dashboard"],
  queryFn: api.dashboard.get
}));

const totalBalanceLabel = computed(() => {
  const minor = dashboardQuery.data?.total_balance_minor;
  if (typeof minor !== "number") return "—";
  return formatGBPMinor(minor);
});

const changePct = computed(() => dashboardQuery.data?.change_vs_last_month_pct);

const changeIsPositive = computed(() => (changePct.value ?? 0) >= 0);

const changeClass = computed(() => {
  if (changePct.value == null) return "text-muted";
  return changeIsPositive.value ? "text-success" : "text-error";
});

const changeIcon = computed(() => {
  if (changePct.value == null) return "i-lucide-minus";
  return changeIsPositive.value ? "i-lucide-arrow-up" : "i-lucide-arrow-down";
});

const changePctLabel = computed(() => {
  if (changePct.value == null) return "—";
  return `${Math.abs(changePct.value).toFixed(1)}%`;
});

const monthlyYieldMinor = computed(() => dashboardQuery.data?.monthly_yield_minor);

const monthlyYieldLabel = computed(() => {
  if (monthlyYieldMinor.value == null) return "—";
  const sign = monthlyYieldMinor.value >= 0 ? "+" : "-";
  return `${sign}${formatGBPMinor(Math.abs(monthlyYieldMinor.value))}`;
});

const monthlyYieldDescriptionClass = computed(() => {
  if (monthlyYieldMinor.value == null) return "text-xl font-bold text-muted whitespace-nowrap";
  return `text-xl font-bold ${monthlyYieldMinor.value >= 0 ? "text-success" : "text-error"} whitespace-nowrap`;
});

const activeAccountsLabel = computed(() => {
  const n = dashboardQuery.data?.active_accounts;
  if (typeof n !== "number") return "—";
  return String(n);
});

const balanceOverTimeOption = computed<ECOption>(() => {
  const points = dashboardQuery.data?.balance_over_time ?? [];
  const dates = points.map((p) => p.date);
  const values = points.map((p) => p.balance_minor / 100);

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
  accountType: AccountTypeName
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
      accountType: r.account_type,
      value: r.balance_minor / 100,
      color: meta.color,
      glow: meta.glow,
      glowEmphasis: meta.glowEmphasis
    };
  });
});

const allocationSelected = ref<Record<string, boolean>>({});

watchEffect(() => {
  for (const d of allocationData.value) {
    if (allocationSelected.value[d.accountType] === undefined) {
      allocationSelected.value[d.accountType] = true;
    }
  }
});

const buildBalanceAllocationOption = (selected: Record<string, boolean>, data: AllocationDatum[]): ECOption => {
  const totalVisible = data.reduce((sum, d) => sum + (selected[d.accountType] === false ? 0 : d.value), 0);
  const percentByKind = Object.fromEntries(
    data.map((d) => {
      const value = selected[d.accountType] === false ? 0 : d.value;
      const pct = totalVisible > 0 ? Math.round((value / totalVisible) * 100) : 0;
      return [d.accountType, pct];
    })
  ) as Record<string, number>;

  return {
    backgroundColor: "transparent",
    tooltip: {
      ...darkTooltipBase,
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
      formatter: (kind: string) => {
        const meta = accountTypeMetaLoose(kind);
        const label = meta?.label ?? kind;
        return `${label}  ${percentByKind[kind] ?? 0}%`;
      }
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
        data: data.map((d) => ({
          name: d.accountType,
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

const onAllocationLegendSelectChanged = (params: { selected?: Record<string, boolean> }) => {
  allocationSelected.value = params.selected ?? {};
};

const helloName = ref("");

const helloMutation = proxyRefs(useMutation({
  mutationFn: api.hello.say
}));

const sayHello = () => {
  helloMutation.reset();
  helloMutation.mutate(helloName.value);
};
</script>
