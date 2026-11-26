<template>
  <UPopover
    arrow
    :content="endPopupContent"
    :ui="{ content: 'p-4 w-80' }"
  >
    <UButton
      label="View options"
      icon="i-lucide-sliders-horizontal"
      color="neutral"
      variant="subtle"
      class="relative"
    >
      <template #trailing>
        <UChip v-if="groupingEnabled" class="pointer-events-none absolute top-0 right-0" />
      </template>
    </UButton>

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
          <UChip :show="groupingEnabled" class="w-full" :ui="{ base: 'pointer-events-none' }">
            <USelect
              v-model="groupBy"
              :items="groupByItems"
              class="w-full"
              color="neutral"
              variant="subtle"
            />
          </UChip>
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
      </div>
    </template>
  </UPopover>
</template>

<script lang="ts" setup>
import type { SelectItem } from "@nuxt/ui";
import type { ActivityPeriod } from "~/generated/bindings";

defineProps<{
  groupByItems: SelectItem[]
  activityPeriodItems: SelectItem[]
}>();

const endPopupContent = useAppPopupContent({ align: "end" });

const groupBy = defineModel<AccountGroupBy>("groupBy", { required: true });
const activityPeriod = defineModel<ActivityPeriod>("activityPeriod", { required: true });

const groupingEnabled = computed(() => groupBy.value !== "none");
</script>
