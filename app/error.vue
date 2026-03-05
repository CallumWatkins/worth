<template>
  <Html class="overflow-x-hidden select-none">
    <Body class="font-sans antialiased">
      <UApp
        :locale="locales[locale]"
        :scroll-body="{ padding: false, margin: false }"
      >
        <AppNavbar />
        <UMain class="error-page-enter">
          <UError
            :key="`${error.statusCode}:${error.statusMessage}`"
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
import * as locales from "@nuxt/ui/locale";

defineProps<{
  error: NuxtError
}>();

const { locale } = useI18n();
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
