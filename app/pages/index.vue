<template>
  <UContainer>
    <UPageHeader
      title="Balance Overview"
      description="A summary of your balances across all accounts"
      :links="links"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />
    <UPageBody>
      <UPageCard
        title="Total Current Balance"
        orientation="horizontal"
        variant="subtle"
        :ui="{
          container: 'lg:grid-cols-[1fr_auto]',
          title: 'uppercase text-muted text-xs'
        }"
      >
        <template #description>
          <div>
            <span class="text-[2.5rem] text-4xl font-bold text-default mr-4">£245,890.12</span>
            <span class="inline-flex gap-1 leading-none">
              <UIcon name="i-lucide-arrow-up" class="size-4 text-success" />
              <span class="text-success">2.4%</span>
              <span class="text-muted text-xs self">vs last month</span>
            </span>
          </div>
        </template>
        <div class="flex gap-4">
          <UPageCard
            title="Monthly Yield" description="+£1,240.82" variant="subtle"
            :ui="{
              container: 'lg:grid-cols-[1fr_auto]',
              title: 'text-muted text-xs',
              description: 'text-xl font-bold text-success'
            }"
          />
          <UPageCard
            title="Active Accounts" description="12" variant="subtle"
            :ui="{
              container: 'lg:grid-cols-[1fr_auto]',
              title: 'text-muted text-xs',
              description: 'text-xl font-bold text-default'
            }"
          />
        </div>
      </UPageCard>
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
