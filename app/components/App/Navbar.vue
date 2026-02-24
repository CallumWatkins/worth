<template>
  <UHeader
    mode="slideover"
    :ui="{
      left: 'lg:flex-none pr-4',
      center: 'h-full'
    }"
  >
    <template #left>
      <NuxtLink
        to="/"
        class="flex items-center gap-x-2"
        aria-label="Go to dashboard"
      >
        <div class="bg-primary rounded-md w-8 h-8">
          <img
            src="/logo.png"
            alt="Worth"
            class="size-8 grayscale brightness-175"
          >
        </div>
        Worth
      </NuxtLink>
    </template>
    <UNavigationMenu
      :items="items"
      highlight
      variant="pill"
      :ui="{
        list: 'h-full gap-x-3',
        root: 'h-full [&>div]:h-full',
        item: 'h-full py-0',
        link: 'h-full py-0 after:bottom-0 before:rounded-none after:inset-x-px',
        viewportWrapper: 'hidden'
      }"
    />
    <template #body>
      <UNavigationMenu
        :items="items"
        orientation="vertical"
        highlight
        variant="link"
        :ui="{
          link: 'py-3'
        }"
      />
    </template>
    <template #right>
      <AppSearchBar class="w-72" />
    </template>
  </UHeader>
</template>

<script lang="ts" setup>
import type { NavigationMenuItem } from "@nuxt/ui";

const items = ref<NavigationMenuItem[]>([
  {
    label: "Dashboard",
    icon: "i-lucide-layout-dashboard",
    to: "/"
  },
  {
    label: "Accounts",
    icon: "i-lucide-wallet",
    to: "/accounts"
  },
  {
    label: "Institutions",
    icon: "i-lucide-building-2",
    to: "/institutions"
  },
  {
    label: "Settings",
    icon: "i-lucide-settings",
    to: "/settings"
  }
]);
</script>
