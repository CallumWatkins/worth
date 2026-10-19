<template>
  <VChart
    :option="option"
    autoresize
    :style="{ height: `${chartHeight}px`, width: '100%' }"
  />
</template>

<script lang="ts" setup>
import type { DashboardBalancePointDto } from "~/generated/bindings";

const props = defineProps<{ points: DashboardBalancePointDto[] }>();
const settings = useSettings();
const { formatCurrency, formatShortDate } = useLocaleFormatters();

const chartHeight = 300;
const lineWidth = 5;
const positive = {
  line: "#22c55e",
  fill: "rgba(34, 197, 94, 0.5)",
  transparent: "rgba(34, 197, 94, 0)"
};
const negative = {
  line: "#ef4444",
  fill: "rgba(239, 68, 68, 0.5)",
  transparent: "rgba(239, 68, 68, 0)"
};

const option = computed<ECOption>(() => {
  const values = props.points.map<[string, number]>((point) => [point.date, convertCurrencyMinorUnitsToMajorAmount(point.balance_minor)]);
  const minimum = values.reduce((min, [, balance]) => Math.min(min, balance), values[0]?.[1] ?? 0);
  const maximum = values.reduce((max, [, balance]) => Math.max(max, balance), values[0]?.[1] ?? 0);
  const padding = (maximum - minimum || Math.abs(minimum) || 1) * 0.1;
  const valuePerPixel = (maximum - minimum + padding * 2) / chartHeight;
  // Leave the zero stroke fully green, then blend from green to red over 25px.
  const greenPadding = valuePerPixel * lineWidth * 0.75;
  const transitionLength = valuePerPixel * 25;

  const colorStops = [
    { offset: 0, color: maximum > 0 ? positive.fill : negative.transparent },
    { offset: 1, color: minimum < 0 ? negative.fill : positive.transparent }
  ];
  if (minimum < 0 && maximum > 0) {
    const zeroOffset = maximum / (maximum - minimum);
    colorStops.splice(1, 0, { offset: zeroOffset, color: positive.transparent }, { offset: zeroOffset, color: negative.transparent });
  }

  return {
    backgroundColor: "transparent",
    visualMap: {
      show: false,
      type: "continuous",
      seriesIndex: 0,
      dimension: 1,
      min: -greenPadding - transitionLength,
      max: -greenPadding,
      inRange: { color: [negative.line, positive.line] }
    },
    tooltip: {
      ...DARK_CHART_TOOLTIP,
      trigger: "axis",
      axisPointer: {
        type: "line",
        lineStyle: { color: "rgba(244, 244, 245, 0.25)" },
        label: {
          formatter: ({ value }) => formatShortDate(new Date(value))
        }
      },
      valueFormatter: (value) => formatCurrency(value as number, settings.value.default_display_currency_code)
    },
    grid: { left: 0, right: 0, top: 0, bottom: 0 },
    xAxis: {
      type: "time",
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { show: false }
    },
    yAxis: {
      show: false,
      scale: true,
      min: minimum - padding,
      max: maximum + padding,
      splitLine: { show: false }
    },
    series: [
      {
        name: "Balance",
        type: "line",
        silent: true,
        // Keep tooltip markers solid while the line uses the visualMap gradient.
        data: values.map((value) => ({
          value,
          visualMap: false,
          itemStyle: { color: value[1] < 0 ? negative.line : positive.line }
        })),
        smooth: true,
        showSymbol: false,
        lineStyle: {
          width: lineWidth,
          shadowBlur: 10,
          shadowColor: "rgba(148, 163, 184, 0.35)",
          shadowOffsetX: 0,
          shadowOffsetY: 0
        },
        areaStyle: {
          origin: 0,
          color: {
            type: "linear",
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops
          }
        }
      }
    ]
  };
});
</script>
