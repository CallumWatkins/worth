<template>
  <UModal
    :title="title"
    :description="description"
    :dismissible="false"
    :close="false"
    :ui="{ footer: 'justify-end' }"
  >
    <template #footer>
      <UButton
        color="neutral"
        variant="outline"
        @click="emit('close', false)"
      >
        {{ cancelLabel }}
      </UButton>
      <UButton
        color="error"
        @click="emit('close', true)"
      >
        {{ confirmLabel }}
      </UButton>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
withDefaults(defineProps<{
  title: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
}>(), {
  description: undefined,
  confirmLabel: "Discard changes",
  cancelLabel: "Stay on page"
});

const emit = defineEmits<{
  close: [value: boolean]
}>();
</script>
