<template>
  <Html :lang="lang" :dir="dir" class="overflow-x-hidden select-none">
    <Body class="font-sans antialiased">
      <UApp
        :locale="uiLocale"
        :scroll-body="{ padding: false, margin: false }"
      >
        <AppNavbar />
        <UMain class="error-page-enter">
          <UError
            :key="`${error.status}:${error.statusText}`"
            :error="error"
            redirect="/"
            :clear="{
              label: 'Back to Dashboard'
            }"
          />
        </UMain>
      </UApp>
    </Body>
  </Html>
</template>

<script setup lang="ts">
import type { NuxtError } from "#app";
import { useAppLocale } from "~/composables/useAppLocale";

defineProps<{
  error: NuxtError
}>();

const { dir, lang, uiLocale } = useAppLocale();
</script>

<style scoped>
.error-page-enter {
  animation: error-page-fade-in 200ms ease-in-out both;
}

@keyframes error-page-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
