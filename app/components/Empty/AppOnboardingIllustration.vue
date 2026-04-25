<template>
  <div class="relative mx-auto flex h-full w-full max-w-[29rem] items-center px-1 py-2">
    <div
      ref="shellRef"
      class="relative w-full overflow-hidden rounded-[2rem] border border-default bg-zinc-950/95 p-4 shadow-2xl transition-[height] duration-300 ease-out"
      :style="shellStyle"
    >
      <div class="relative h-full">
        <div
          class="h-full w-full transition-opacity duration-200 ease-out"
          :class="step === 0 ? 'relative z-10 opacity-100' : 'pointer-events-none absolute inset-0 opacity-0'"
        >
          <div :ref="setPaneRef(0)" class="space-y-4">
            <div class="grid gap-4 grid-cols-[1.05fr_0.95fr]">
              <div class="rounded-2xl border border-primary/25 bg-primary/10 p-4">
                <div class="text-xs uppercase tracking-wide text-primary/80">
                  Net worth
                </div>
                <div class="mt-2 text-2xl font-semibold text-white">
                  £148,710.36
                </div>
                <div class="mt-1 text-sm text-primary/80">
                  +1.9% this month
                </div>
              </div>

              <div class="rounded-2xl border border-default bg-zinc-900 p-4">
                <div class="mb-3 text-xs uppercase tracking-wide text-zinc-400">
                  Monthly Yield
                </div>
                <div class="text-xl font-semibold text-success">
                  +£2,758.22
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-default bg-zinc-900 p-4">
              <div class="mb-3.5 text-xs uppercase tracking-wide text-zinc-400">
                Portfolio Allocation
              </div>
              <div class="flex items-center gap-10 ms-2 me-4">
                <svg viewBox="0 0 100 100" class="h-30 w-30 shrink-0 overflow-visible" aria-hidden="true">
                  <defs>
                    <filter id="allocation-cap-shadow" x="-50%" y="-50%" width="200%" height="200%">
                      <feGaussianBlur in="SourceGraphic" stdDeviation="0.8" />
                    </filter>
                    <clipPath
                      v-for="segment in allocationSegments"
                      :id="segment.capClipId"
                      :key="segment.capClipId"
                      clipPathUnits="userSpaceOnUse"
                    >
                      <rect
                        x="0"
                        y="-16"
                        width="20"
                        height="32"
                        :transform="`translate(${segment.capX} ${segment.capY}) rotate(${segment.capShadowAngle})`"
                      />
                    </clipPath>
                  </defs>
                  <g style="filter: drop-shadow(0 0 4px rgb(0 0 0 / 0.3));">
                    <circle cx="50" cy="50" r="39" fill="none" stroke="rgb(39 39 42)" stroke-width="14" />
                    <circle
                      v-for="segment in allocationSegments"
                      :key="segment.accountType"
                      cx="50"
                      cy="50"
                      r="39"
                      fill="none"
                      :stroke="segment.color"
                      stroke-width="14"
                      stroke-linecap="butt"
                      pathLength="100"
                      :stroke-dasharray="`${segment.percent} ${100 - segment.percent}`"
                      :stroke-dashoffset="segment.offset"
                      :transform="`rotate(${allocationRotationDeg - 90} 50 50)`"
                    />
                    <g
                      v-for="segment in allocationSegments"
                      :key="`${segment.accountType}-cap-shadow`"
                      :clip-path="`url(#${segment.capClipId})`"
                    >
                      <circle
                        :cx="segment.capShadowX"
                        :cy="segment.capShadowY"
                        r="6.7"
                        fill="rgb(0 0 0)"
                        fill-opacity="0.6"
                        filter="url(#allocation-cap-shadow)"
                      />
                    </g>
                    <circle
                      v-for="segment in allocationSegments"
                      :key="`${segment.accountType}-cap`"
                      :cx="segment.capX"
                      :cy="segment.capY"
                      r="7"
                      :fill="segment.color"
                    />
                  </g>
                </svg>

                <div class="min-w-0 flex-1 space-y-2 text-xs text-zinc-300">
                  <div
                    v-for="item in allocationSegments"
                    :key="item.accountType"
                    class="flex items-center justify-between gap-3"
                  >
                    <div class="flex min-w-0 items-center gap-2">
                      <span class="size-2 rounded-full" :style="{ backgroundColor: item.color }" />
                      <span>{{ item.label }}</span>
                    </div>
                    <span class="text-muted">{{ item.percent }}%</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          class="h-full w-full transition-opacity duration-200 ease-out"
          :class="step === 1 ? 'relative z-10 opacity-100' : 'pointer-events-none absolute inset-0 opacity-0'"
        >
          <div :ref="setPaneRef(1)" class="space-y-4">
            <div class="rounded-2xl border border-[#00AEEF]/18 bg-[#00AEEF]/4 p-4">
              <div class="flex items-start justify-between gap-4">
                <div class="text-lg font-semibold text-white">
                  Barclays
                </div>
                <div class="rounded-full bg-[#00AEEF]/10 px-3 py-1 text-xs font-medium text-[#CBEFFC]">
                  2 accounts
                </div>
              </div>
              <div class="mt-4 space-y-2">
                <div class="flex items-center justify-between rounded-xl bg-[#00AEEF]/10 px-3 py-3 text-sm text-zinc-100">
                  <span>Premier Current</span>
                  <span>£2,920.65</span>
                </div>
                <div class="flex items-center justify-between rounded-xl bg-[#00AEEF]/10 px-3 py-3 text-sm text-zinc-100">
                  <span>Blue Rewards Saver</span>
                  <span>£25,580.89</span>
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-[#C8102E]/18 bg-[#C8102E]/4 p-4">
              <div class="flex items-start justify-between gap-4">
                <div class="text-lg font-semibold text-white">
                  Vanguard
                </div>
                <div class="rounded-full bg-[#C8102E]/10 px-3 py-1 text-xs font-medium text-[#FFD6DD]">
                  2 accounts
                </div>
              </div>
              <div class="mt-4 space-y-2">
                <div class="flex items-center justify-between rounded-xl bg-[#C8102E]/10 px-3 py-3 text-sm text-zinc-100">
                  <span>Stocks & Shares ISA</span>
                  <span>£62,554.21</span>
                </div>
                <div class="flex items-center justify-between rounded-xl bg-[#C8102E]/10 px-3 py-3 text-sm text-zinc-100">
                  <span>SIPP</span>
                  <span>£28,400.70</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          class="h-full w-full transition-opacity duration-200 ease-out"
          :class="step === 2 ? 'relative z-10 opacity-100' : 'pointer-events-none absolute inset-0 opacity-0'"
        >
          <div :ref="setPaneRef(2)" class="space-y-4">
            <div class="flex items-center justify-between rounded-2xl bg-zinc-900 px-4 py-3">
              <div>
                <div class="text-sm font-semibold text-zinc-100">
                  Premier Current
                </div>
                <div class="text-xs text-zinc-400">
                  Barclays
                </div>
              </div>
              <div class="text-right">
                <div class="text-xs text-zinc-400">
                  Latest balance
                </div>
                <div class="text-sm font-medium text-zinc-100">
                  £2,920.65
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-default bg-zinc-900 p-4">
              <div class="mb-3 text-xs uppercase tracking-wide text-zinc-400">
                Balance over time
              </div>
              <div class="relative h-14 overflow-hidden rounded-xl bg-zinc-900/45">
                <svg viewBox="0 0 320 48" class="h-full w-full" aria-hidden="true">
                  <path d="M18 40C39 40 51 33 73 30C94 27 108 35 131 25C154 15 173 8 195 14C218 20 239 11 261 9C278 7 291 9 302 5" fill="none" stroke="rgb(34 197 94)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
                  <circle cx="18" cy="40" r="3.25" fill="rgb(34 197 94)" />
                  <circle cx="73" cy="30" r="3.25" fill="rgb(34 197 94)" />
                  <circle cx="131" cy="25" r="3.25" fill="rgb(34 197 94)" />
                  <circle cx="195" cy="14" r="3.25" fill="rgb(34 197 94)" />
                  <circle cx="261" cy="9" r="3.25" fill="rgb(34 197 94)" />
                  <circle cx="302" cy="5" r="3.25" fill="rgb(34 197 94)" />
                </svg>
              </div>
            </div>

            <div class="rounded-2xl border border-default bg-zinc-900 p-4">
              <div class="mb-3 text-xs uppercase tracking-wide text-zinc-400">
                Snapshots
              </div>
              <div class="space-y-1.5">
                <div class="grid grid-cols-[1fr_auto] gap-3 rounded-xl bg-zinc-800/80 px-3 py-2.5 text-[13px] text-zinc-100">
                  <span>{{ formatShortDate(`${previousSnapshotMonth}-07`) }}</span>
                  <span>£2,400.10</span>
                </div>
                <div class="grid grid-cols-[1fr_auto] gap-3 rounded-xl bg-zinc-800/80 px-3 py-2.5 text-[13px] text-zinc-100">
                  <span>{{ formatShortDate(`${previousSnapshotMonth}-10`) }}</span>
                  <span>£2,610.29</span>
                </div>
                <div class="grid grid-cols-[1fr_auto] gap-3 rounded-xl bg-primary/10 px-3 py-2.5 text-[13px] text-zinc-100 ring-1 ring-primary/25">
                  <span>{{ formatShortDate(`${previousSnapshotMonth}-12`) }}</span>
                  <span>£2,920.65</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          class="h-full w-full transition-opacity duration-200 ease-out"
          :class="step === 3 ? 'relative z-10 opacity-100' : 'pointer-events-none absolute inset-0 opacity-0'"
        >
          <div class="flex h-full items-center justify-center">
            <div :ref="setPaneRef(3)" class="flex items-center justify-center min-h-[19rem]">
              <svg viewBox="0 0 260 260" class="h-64 w-64" aria-hidden="true">
                <defs>
                  <linearGradient id="rocket-body" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stop-color="rgb(250 250 250)" />
                    <stop offset="100%" stop-color="rgb(212 212 216)" />
                  </linearGradient>
                  <linearGradient id="rocket-flame" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stop-color="rgb(253 224 71)" />
                    <stop offset="55%" stop-color="rgb(251 146 60)" />
                    <stop offset="100%" stop-color="rgb(239 68 68)" />
                  </linearGradient>
                </defs>

                <g class="onboarding-rocket-orbit-outer" :class="{ 'is-active': step === 3 }">
                  <circle cx="130" cy="130" r="118" fill="rgb(34 197 94 / 0.07)" />
                </g>

                <g class="onboarding-rocket-orbit-inner" :class="{ 'is-active': step === 3 }">
                  <circle cx="130" cy="130" r="88" fill="rgb(34 197 94 / 0.05)" />
                </g>

                <g class="onboarding-rocket-ship" :class="{ 'is-active': step === 3 }">
                  <g transform="rotate(18 130 130)">
                    <path d="M136 44C171 69 181 119 156 164L129 154L102 164C77 119 87 69 122 44C126 41 132 41 136 44Z" fill="url(#rocket-body)" stroke="rgb(82 82 91)" stroke-width="3" />
                    <circle cx="129" cy="92" r="16" fill="rgb(8 47 73)" stroke="rgb(125 211 252)" stroke-width="3" />
                    <path d="M102 164L82 188L107 182L121 157Z" fill="rgb(34 197 94 / 0.75)" stroke="rgb(21 128 61)" stroke-width="3" stroke-linejoin="round" />
                    <path d="M156 164L176 188L151 182L137 157Z" fill="rgb(34 197 94 / 0.75)" stroke="rgb(21 128 61)" stroke-width="3" stroke-linejoin="round" />
                    <path d="M118 160L129 220L140 160Z" fill="url(#rocket-flame)" />
                    <path d="M123 164L129 196L135 164Z" fill="rgb(254 249 195 / 0.95)" />
                  </g>
                </g>

                <g class="onboarding-rocket-stars" :class="{ 'is-active': step === 3 }">
                  <circle cx="64" cy="64" r="4" fill="rgb(250 250 250 / 0.85)" />
                  <circle cx="198" cy="58" r="3" fill="rgb(250 250 250 / 0.65)" />
                  <circle cx="211" cy="142" r="4" fill="rgb(250 250 250 / 0.75)" />
                  <circle cx="54" cy="172" r="3" fill="rgb(250 250 250 / 0.7)" />
                  <circle cx="185" cy="198" r="2.5" fill="rgb(250 250 250 / 0.65)" />
                </g>
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { ComponentPublicInstance } from "vue";
import type { AccountTypeName } from "~/generated/bindings";
import { ACCOUNT_TYPE_META } from "~/utils/account-type-meta";

const props = defineProps<{
  step: number
}>();

const { formatShortDate } = useLocaleFormatters();
const paneRefs = ref<Array<HTMLElement | null>>([]);
const shellHeight = ref<number | null>(null);
const shellRef = ref<HTMLElement | null>(null);
const allocationRotationDeg = -4;
const allocationRotationRad = allocationRotationDeg * (Math.PI / 180);
const allocation = [
  { accountType: "current", percent: 8 },
  { accountType: "savings", percent: 12 },
  { accountType: "isa", percent: 29 },
  { accountType: "investment", percent: 15 },
  { accountType: "pension", percent: 36 }
] as const satisfies Array<{ accountType: AccountTypeName, percent: number }>;
let allocationOffset = 0;
const allocationSegments = allocation.map(({ accountType, percent }) => {
  const endAngle = ((allocationOffset + percent) / 100) * Math.PI * 2 - (Math.PI / 2) + allocationRotationRad;
  const tangentX = -Math.sin(endAngle);
  const tangentY = Math.cos(endAngle);
  const capShadowAngle = Math.atan2(tangentY, tangentX) * (180 / Math.PI);
  const segment = {
    accountType,
    label: ACCOUNT_TYPE_META[accountType].label,
    color: ACCOUNT_TYPE_META[accountType].color,
    percent,
    offset: -allocationOffset,
    capX: 50 + (39 * Math.cos(endAngle)),
    capY: 50 + (39 * Math.sin(endAngle)),
    capShadowX: 50 + (39 * Math.cos(endAngle)) + (tangentX * 1.15),
    capShadowY: 50 + (39 * Math.sin(endAngle)) + (tangentY * 1.15),
    capShadowAngle,
    capClipId: `onboarding-allocation-cap-clip-${accountType}`
  };
  allocationOffset += percent;
  return segment;
});
const previousSnapshotMonth = (() => {
  const previousMonth = new Date();
  previousMonth.setMonth(previousMonth.getMonth() - 1);
  return `${previousMonth.getFullYear()}-${String(previousMonth.getMonth() + 1).padStart(2, "0")}`;
})();

let resizeObserver: ResizeObserver | null = null;

const shellStyle = computed(() => {
  if (shellHeight.value == null) return undefined;
  return { height: `${shellHeight.value}px` };
});

function setPaneRef(index: number) {
  return (el: Element | ComponentPublicInstance | null) => {
    paneRefs.value[index] = el instanceof HTMLElement ? el : null;
  };
}

function updateShellHeight() {
  const nextHeight = paneRefs.value[props.step]?.offsetHeight ?? 0;
  if (nextHeight > 0) {
    const shellStyles = shellRef.value ? getComputedStyle(shellRef.value) : null;
    const paddingTop = Number.parseFloat(shellStyles?.paddingTop ?? "0") || 0;
    const paddingBottom = Number.parseFloat(shellStyles?.paddingBottom ?? "0") || 0;
    const borderTop = Number.parseFloat(shellStyles?.borderTopWidth ?? "0") || 0;
    const borderBottom = Number.parseFloat(shellStyles?.borderBottomWidth ?? "0") || 0;

    shellHeight.value = Math.ceil(nextHeight + paddingTop + paddingBottom + borderTop + borderBottom);
  }
}

async function syncShellHeight() {
  await nextTick();
  updateShellHeight();
}

watch(() => props.step, async () => {
  await syncShellHeight();
});

onMounted(async () => {
  await nextTick();
  updateShellHeight();

  if (typeof ResizeObserver !== "undefined") {
    resizeObserver = new ResizeObserver(() => {
      updateShellHeight();
    });

    paneRefs.value.forEach((pane) => {
      if (pane) {
        resizeObserver?.observe(pane);
      }
    });
  }

  window.addEventListener("resize", updateShellHeight);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  window.removeEventListener("resize", updateShellHeight);
});
</script>
