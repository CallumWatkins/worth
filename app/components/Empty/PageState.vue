<template>
  <UEmpty
    :icon="icon"
    :title="title"
    :description="description"
    size="xl"
    variant="naked"
    :ui="{
      root: 'mx-auto max-w-xl',
      title: 'text-2xl',
      description: 'text-toned',
      actions: 'pt-2'
    }"
  >
    <template #actions>
      <UButton
        v-if="actionLabel"
        :icon="actionIcon"
        size="lg"
        @click="$emit('action')"
      >
        {{ actionLabel }}
      </UButton>
    </template>
  </UEmpty>
</template>

<script lang="ts" setup>
withDefaults(defineProps<{
  title: string
  description: string
  icon: string
  actionLabel?: string
  actionIcon?: string
}>(), {
  actionLabel: undefined,
  actionIcon: undefined
});

defineEmits<{
  action: []
}>();
</script>
