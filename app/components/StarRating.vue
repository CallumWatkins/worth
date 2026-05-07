<template>
  <div
    class="flex items-center"
    role="group"
    @mouseleave="hoveredRating = null"
  >
    <button
      v-for="score in max"
      :key="score"
      type="button"
      class="relative flex size-11 cursor-pointer items-center justify-center overflow-visible transition-colors duration-200 ease-out focus:outline-none focus-visible:ring-2 focus-visible:ring-warning"
      :class="getRatingButtonClass(score)"
      :aria-pressed="modelValue === score"
      :aria-label="`${score} out of ${max}`"
      @click="modelValue = modelValue === score ? null : score"
      @mouseenter="hoveredRating = score"
      @focus="hoveredRating = score"
      @blur="hoveredRating = null"
    >
      <span
        aria-hidden="true"
        class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_center,var(--ui-warning)_0%,transparent_68%)] blur-md transition-opacity duration-200 ease-out"
        :class="getRatingGlowClass(score)"
      />
      <svg
        viewBox="0 0 24 24"
        aria-hidden="true"
        class="relative size-8"
        :class="getRatingIconClass(score)"
      >
        <path
          fill="currentColor"
          class="transition-opacity duration-200 ease-out"
          :class="isRatingStarFilled(score) ? 'opacity-100' : 'opacity-0'"
          d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
        />
        <path
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
        />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  max: number
}>();

const modelValue = defineModel<number | null>({ required: true });
const hoveredRating = ref<number | null>(null);

function getRatingButtonClass(score: number) {
  const selected = modelValue.value != null && score <= modelValue.value;
  const previewed = hoveredRating.value != null && score <= hoveredRating.value;

  if (selected || previewed) return "text-warning";
  return "text-muted";
}

function isRatingStarFilled(score: number) {
  return (modelValue.value != null && score <= modelValue.value) || (hoveredRating.value != null && score <= hoveredRating.value);
}

function getRatingIconClass(score: number) {
  const selected = modelValue.value != null && score <= modelValue.value;
  const previewed = hoveredRating.value != null && score <= hoveredRating.value;

  if (hoveredRating.value == null || (selected && previewed)) return "transition-opacity";
  if ((selected && !previewed) || (!selected && previewed)) return "opacity-85 transition-opacity";
  return "transition-opacity";
}

function getRatingGlowClass(score: number) {
  const selected = modelValue.value != null && score <= modelValue.value;
  const insideHoverPreview = hoveredRating.value == null || score <= hoveredRating.value;

  return selected && insideHoverPreview ? "opacity-30" : "opacity-0";
}
</script>
