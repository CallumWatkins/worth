<template>
  <UHeader
    v-model:open="mobileNavOpen"
    mode="slideover"
    :ui="{
      left: 'lg:flex-none pr-4',
      center: 'h-full'
    }"
  >
    <template #left>
      <NuxtLink
        :to="{ name: 'index' }"
        aria-label="Go to dashboard"
        tabindex="-1"
      >
        <img
          :src="colorMode.value === 'dark' ? logoDarkTheme : logoLightTheme"
          alt="Worth"
          class="h-10"
        >
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
        link: [
          'h-full py-0 after:bottom-0 before:rounded-none after:inset-x-px',
          'focus-visible:before:bg-elevated/50 focus-visible:not-data-active:text-highlighted',
          'not-forced-colors:focus-visible:before:outline-none'
        ],
        linkLeadingIcon: 'group-[&:not([data-active]):focus-visible]:text-default',
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
      <AppSearchBar class="lg:w-72" />
    </template>
  </UHeader>
</template>

<script lang="ts" setup>
import type { NavigationMenuItem } from "@nuxt/ui";
import logoDarkTheme from "~/assets/worth_combo_mark_gradient_green.svg";
import logoLightTheme from "~/assets/worth_combo_mark_gradient_green_dark.svg";

const colorMode = useColorMode();
const mobileNavOpen = ref(false);

useNavigationLayer({
  id: "mobile-navigation-slideover",
  open: mobileNavOpen,
  close: () => {
    mobileNavOpen.value = false;
  }
});

const items: NavigationMenuItem[] = [
  {
    label: "Dashboard",
    icon: "i-lucide-layout-dashboard",
    to: { name: "index" }
  },
  {
    label: "Accounts",
    icon: "i-lucide-wallet",
    to: { name: "accounts" }
  },
  {
    label: "Institutions",
    icon: "i-lucide-building-2",
    to: { name: "institutions" }
  },
  {
    label: "Settings",
    icon: "i-lucide-settings",
    to: { name: "settings" }
  }
];
</script>
