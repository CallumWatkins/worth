<template>
  <Html class="overflow-x-hidden select-none">
    <Body class="font-sans antialiased">
      <UApp
        :locale="locales[locale]"
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
import * as locales from "@nuxt/ui/locale";

const { locale } = useI18n();
</script>
