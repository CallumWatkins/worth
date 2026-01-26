<template>
  <UContainer>
    <UPageHeader
      title="Accounts"
      description="Manage your accounts and their balances"
      :links="links"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />
    <UPageBody class="space-y-8">
      TODO
    </UPageBody>
  </UContainer>
</template>

<script lang="ts" setup>
import type { ButtonProps } from "@nuxt/ui";

const links = ref<ButtonProps[]>([
  {
    label: "Export Data",
    icon: "i-lucide-download",
    to: "/data/export",
    variant: "subtle"
  },
  {
    label: "Add New Account",
    icon: "i-lucide-plus",
    to: "/accounts/new",
    color: "primary",
    variant: "solid"
  }
]);
</script>
