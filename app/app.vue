<template>
  <Html :lang="lang" :dir="dir" class="overflow-x-hidden select-none">
    <Body class="font-sans antialiased">
      <UApp
        :locale="uiLocale"
        :scroll-body="{ padding: false, margin: false }"
      >
        <AppNavbar />
        <UMain>
          <NuxtPage />
        </UMain>
      </UApp>
    </Body>
  </Html>
</template>

<script setup lang="ts">
import { useAppLocale } from "~/composables/useAppLocale";

const { dir, lang, uiLocale } = useAppLocale();
</script>
