<template>
  <AccountsFilterMenu
    v-model="states"
    label="Balance"
    content-class="max-h-(--reka-combobox-content-available-height) overflow-y-auto"
    :items="items"
    :active="range.minimum !== null || range.maximum !== null"
    @clear="range = createAccountBalanceRange()"
  >
    <template #content-bottom>
      <div
        class="border-t border-default p-3 space-y-3"
        @keydown="!['Tab', 'Escape'].includes($event.key) && $event.stopPropagation()"
      >
        <UFormField>
          <UFieldGroup class="w-full">
            <USelect
              v-model="range.minimumOperator"
              :items="minimumOperators"
              :trailing="false"
              aria-label="Minimum comparison"
              :ui="{ itemTrailing: 'hidden' }"
              class="w-8 shrink-0 justify-center"
            />
            <UInputNumber
              :model-value="range.minimum"
              :increment="false"
              :decrement="false"
              :step-snapping="false"
              :format-options="{ maximumFractionDigits: 2 }"
              aria-label="Minimum balance"
              class="w-36"
              @update:model-value="range.minimum = $event ?? null"
            />
          </UFieldGroup>
        </UFormField>
        <UFormField>
          <UFieldGroup class="w-full">
            <USelect
              v-model="range.maximumOperator"
              :items="maximumOperators"
              :trailing="false"
              aria-label="Maximum comparison"
              :ui="{ itemTrailing: 'hidden' }"
              class="w-8 shrink-0 justify-center"
            />
            <UInputNumber
              :model-value="range.maximum"
              :increment="false"
              :decrement="false"
              :step-snapping="false"
              :format-options="{ maximumFractionDigits: 2 }"
              aria-label="Maximum balance"
              class="w-36"
              @update:model-value="range.maximum = $event ?? null"
            />
          </UFieldGroup>
        </UFormField>
      </div>
    </template>
  </AccountsFilterMenu>
</template>

<script lang="ts" setup>
defineProps<{
  items: { label: string, value: "active" | "empty" }[]
}>();

const states = defineModel<AccountFilters["balances"]>({ required: true });
const range = defineModel<AccountBalanceRange>("range", { required: true });

const minimumOperators = [
  { label: "≥", value: ">=" as const },
  { label: ">", value: ">" as const }
];
const maximumOperators = [
  { label: "≤", value: "<=" as const },
  { label: "<", value: "<" as const }
];
</script>
