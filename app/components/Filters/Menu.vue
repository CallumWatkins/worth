<template>
  <UChip :show="model.length > 0 || active">
    <USelectMenu
      :model-value="model as (string | number)[]"
      :items="items as MenuItems"
      value-key="value"
      multiple
      :aria-label="label"
      :search-input="false"
      :content="{ align: 'start' }"
      color="neutral"
      variant="soft"
      :ui="{
        content: ['w-max max-w-[calc(100vw-2rem)]', contentClass],
        empty: !items?.length ? 'hidden' : undefined,
        itemTrailing: 'min-w-5 shrink-0'
      }"
      @update:model-value="model = $event as T[]"
    >
      <template #default>
        {{ label }}
      </template>
      <template #item-label="{ item }">
        <slot name="item-label" :item="item">
          {{ item.label }}
        </slot>
      </template>
      <template #content-bottom>
        <slot name="content-bottom" />
        <div class="border-t border-default p-1">
          <UButton
            label="Clear selection"
            color="neutral"
            variant="link"
            size="sm"
            :disabled="model.length === 0 && !active"
            @click.stop="model = []; emit('clear')"
          />
        </div>
      </template>
    </USelectMenu>
  </UChip>
</template>

<script lang="ts" setup generic="T extends string | number">
import type { ChipProps } from "@nuxt/ui";

interface FilterItem<V> {
  label: string
  value: V
  chip?: ChipProps
}

type MenuItems = FilterItem<string | number>[] | FilterItem<string | number>[][];

defineProps<{
  label: string
  active?: boolean
  contentClass?: string
  items?: FilterItem<T>[] | FilterItem<T>[][]
}>();

const emit = defineEmits<{ clear: [] }>();

const model = defineModel<T[]>({ default: () => [] });
</script>
