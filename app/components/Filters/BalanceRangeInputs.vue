<template>
  <div
    class="p-3 space-y-3"
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
          @input="range.minimum = parseCurrencyInputNumberEventValue($event) ?? null"
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
          @input="range.maximum = parseCurrencyInputNumberEventValue($event) ?? null"
          @update:model-value="range.maximum = $event ?? null"
        />
      </UFieldGroup>
    </UFormField>
  </div>
</template>

<script lang="ts" setup>
const range = defineModel<BalanceRange>({ required: true });

const minimumOperators = [
  { label: "≥", value: ">=" as const },
  { label: ">", value: ">" as const }
];
const maximumOperators = [
  { label: "≤", value: "<=" as const },
  { label: "<", value: "<" as const }
];
</script>
