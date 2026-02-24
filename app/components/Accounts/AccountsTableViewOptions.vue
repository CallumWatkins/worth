<template>
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
            :content="{ bodyLock: false }"
          />
        </UFormField>

        <UFormField name="activityPeriod" label="Activity period">
          <USelect
            v-model="activityPeriod"
            :items="activityPeriodItems"
            class="w-full"
            color="neutral"
            variant="subtle"
            :content="{ bodyLock: false }"
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
</template>

<script lang="ts" setup>
import type { SelectItem } from "@nuxt/ui";
import type { ActivityPeriod } from "~/generated/bindings";

defineProps<{
  groupByItems: SelectItem[]
  activityPeriodItems: SelectItem[]
}>();

const groupBy = defineModel<AccountGroupBy>("groupBy", { required: true });
const activityPeriod = defineModel<ActivityPeriod>("activityPeriod", { required: true });
const showEmpty = defineModel<boolean>("showEmpty", { required: true });
</script>
