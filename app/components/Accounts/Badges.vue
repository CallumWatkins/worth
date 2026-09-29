<template>
  <div v-if="badges.length" class="flex items-center gap-1.5" :class="limit === undefined ? 'flex-wrap' : 'shrink-0'">
    <UBadge
      v-for="badge in visibleBadges"
      :key="badge.key"
      :color="badge.closed ? 'warning' : 'neutral'"
      :variant="badge.closed ? 'subtle' : 'soft'"
      :class="limit === undefined ? 'max-w-full' : 'shrink-0'"
      :title="badge.description ?? undefined"
    >
      <span :class="limit === undefined ? 'whitespace-normal wrap-anywhere' : 'whitespace-nowrap'">{{ badge.name }}</span>
    </UBadge>
    <UPopover v-if="hiddenCount > 0" :content="{ align: 'start' }">
      <UButton
        color="neutral"
        variant="soft"
        size="xs"
        class="tabular-nums"
        :title="`${hiddenCount} more ${hiddenCount === 1 ? 'label' : 'labels'}`"
        :aria-label="`Show ${hiddenCount} more badges for ${accountName}`"
        @click.stop
        @keydown.stop
      >
        +{{ hiddenCount }}
      </UButton>
      <template #content>
        <div class="flex flex-col items-start gap-1.5 max-w-xs p-3" @click.stop @keydown.stop>
          <UBadge
            v-for="badge in hiddenBadges"
            :key="badge.key"
            :color="badge.closed ? 'warning' : 'neutral'"
            :variant="badge.closed ? 'subtle' : 'soft'"
            class="max-w-full whitespace-normal wrap-anywhere"
            :title="badge.description ?? undefined"
          >
            {{ badge.name }}
          </UBadge>
        </div>
      </template>
    </UPopover>
  </div>
</template>

<script setup lang="ts">
import type { LabelDto } from "~/generated/bindings";

const props = defineProps<{
  labels: LabelDto[]
  accountName: string
  closedDate?: string | null
  limit?: number
}>();

const { formatShortDate } = useLocaleFormatters();
const badges = computed(() => [
  ...(props.closedDate != null ? [{ key: "closed", name: "Closed", closed: true, description: `Closed on ${formatShortDate(props.closedDate)}` }] : []),
  ...props.labels.toSorted((a, b) => a.name.localeCompare(b.name))
    .map((label) => ({ key: `label-${label.id}`, name: label.name, closed: false, description: label.description }))
]);
const visibleBadges = computed(() => props.limit === undefined ? badges.value : badges.value.slice(0, Math.max(1, props.limit)));
const hiddenBadges = computed(() => badges.value.slice(visibleBadges.value.length));
const hiddenCount = computed(() => hiddenBadges.value.length);
</script>
