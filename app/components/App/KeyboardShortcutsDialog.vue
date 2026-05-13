<template>
  <UModal
    v-model:open="keyboardShortcutsOpen"
    title="Keyboard shortcuts"
  >
    <template #body>
      <div class="divide-y divide-default">
        <div
          v-for="shortcut in shortcuts"
          :key="shortcut.label"
          class="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0"
        >
          <div class="text-sm text-highlighted">
            {{ shortcut.label }}
          </div>

          <div class="flex flex-wrap items-baseline justify-end gap-2">
            <template
              v-for="(combo, comboIndex) in shortcut.combos"
              :key="combo.join('_')"
            >
              <div class="flex items-baseline gap-1">
                <template
                  v-for="(key, keyIndex) in combo"
                  :key="`${combo.join('_')}:${key}:${keyIndex}`"
                >
                  <UKbd :value="key" size="lg" />
                  <span v-if="keyIndex < combo.length - 1">+</span>
                </template>
              </div>
              <span v-if="comboIndex < shortcut.combos.length - 1" class="text-xs text-muted">or</span>
            </template>
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
const { keyboardShortcutsOpen, shortcuts } = useGlobalKeyboardShortcuts(({ keyboardShortcutsOpen }) => [
  {
    label: "Show keyboard shortcuts",
    combos: [["meta", "/"], ["?"]],
    order: 10,
    enabledWhenShortcutsOpen: true,
    handler: () => {
      keyboardShortcutsOpen.value = !keyboardShortcutsOpen.value;
    }
  },
  {
    label: "Open settings",
    combos: [["meta", ","]],
    order: 30,
    handler: async () => {
      await navigateTo({ name: "settings" });
    }
  }
]);
</script>
