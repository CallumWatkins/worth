<template>
  <div class="flex min-h-[calc(100vh-12rem)] items-center justify-center py-12">
    <div class="w-full max-w-5xl">
      <div class="mx-auto max-w-2xl text-center mb-8">
        <div class="text-sm font-medium uppercase tracking-[0.24em] text-primary/80">
          Welcome to Worth
        </div>
        <h1 class="mt-3 font-semibold tracking-tight text-highlighted text-4xl">
          Keep an eye on your money at your own pace, across all your accounts.
        </h1>
      </div>

      <UPageCard
        class="h-[35rem]"
        spotlight
        variant="outline"
        :ui="{
          container: 'overflow-hidden sm:p-4 lg:p-6',
          body: 'p-0 h-full',
          spotlight: 'bg-default/95'
        }"
      >
        <div class="grid h-full min-h-0 gap-0 grid-cols-[minmax(0,1.02fr)_minmax(28rem,0.98fr)] lg:grid-cols-[minmax(0,1.02fr)_minmax(29rem,0.98fr)]">
          <div class="flex h-full min-w-0 flex-col border-default border-b-0 border-r p-8 lg:p-10">
            <UProgress
              color="primary"
              size="sm"
              :model-value="activeStep + 1"
              :max="steps.length"
            />

            <div class="mt-8 flex-1 min-h-[16rem]">
              <Transition :name="contentTransitionName" mode="out-in">
                <div :key="`content-${activeStep}`" class="flex h-full flex-col">
                  <div class="text-sm font-medium text-primary/80">
                    {{ steps[activeStep]!.eyebrow }}
                  </div>
                  <h2 class="mt-3 font-semibold tracking-tight text-highlighted text-3xl">
                    {{ steps[activeStep]!.title }}
                  </h2>
                  <p class="mt-4 max-w-xl leading-7 text-toned">
                    {{ steps[activeStep]!.description }}
                  </p>
                </div>
              </Transition>
            </div>

            <div class="mt-8 flex items-end justify-between gap-3 border-t border-default pt-6">
              <div class="flex min-w-[5.5rem]">
                <UButton
                  color="neutral"
                  variant="ghost"
                  icon="i-lucide-arrow-left"
                  class="transition-opacity duration-200 text-muted"
                  :class="activeStep > 0 ? 'opacity-100' : 'pointer-events-none opacity-0'"
                  @click="goBack"
                >
                  Back
                </UButton>
              </div>

              <div class="flex items-center gap-3">
                <UButton
                  v-if="!isLastStep"
                  color="neutral"
                  variant="ghost"
                  @click="skipToLastStep"
                >
                  Skip
                </UButton>
                <UButton
                  v-if="!isLastStep"
                  trailing-icon="i-lucide-arrow-right"
                  @click="goNext"
                >
                  Next
                </UButton>
                <UButton
                  v-else
                  icon="i-lucide-plus"
                  @click="createAccountOpen = true"
                >
                  Create account
                </UButton>
              </div>
            </div>
          </div>

          <div class="relative flex items-center justify-center overflow-hidden bg-linear-to-br from-primary/8 via-transparent to-primary/4 h-full p-6 lg:p-8">
            <div class="flex h-full w-full items-center justify-center overflow-hidden">
              <EmptyAppOnboardingIllustration :step="activeStep" />
            </div>
          </div>
        </div>
      </UPageCard>

      <AccountsCreateDialog v-model:open="createAccountOpen" />
    </div>
  </div>
</template>

<script lang="ts" setup>
interface OnboardingStep {
  eyebrow: string
  title: string
  description: string
}

const steps: OnboardingStep[] = [
  {
    eyebrow: "What Worth does",
    title: "Track your account balances over time.",
    description: "Worth turns balance updates into a timeline of your net worth, so you can see how your accounts move together over weeks, months, and years."
  },
  {
    eyebrow: "How things stay organised",
    title: "Institutions group related accounts together.",
    description: "Use institutions for banks, brokers, or providers like Barclays or Vanguard, then keep the accounts under each one in a clean structure."
  },
  {
    eyebrow: "How balances are recorded",
    title: "Create a snapshot whenever a balance changes.",
    description: "Add a new snapshot whenever you want to record a balance change. Missing days simply keep the previous value; you don't need daily entries."
  },
  {
    eyebrow: "Ready to begin",
    title: "Add your first account to get started.",
    description: "A clearer view of your wealth starts here."
  }
];

const createAccountOpen = ref(false);
const activeStep = ref(0);
const transitionDirection = ref<"forward" | "backward">("forward");

const isLastStep = computed(() => activeStep.value === steps.length - 1);
const contentTransitionName = computed(() => (
  transitionDirection.value === "forward"
    ? "onboarding-forward"
    : "onboarding-backward"
));

function goBack() {
  if (activeStep.value === 0) return;
  transitionDirection.value = "backward";
  activeStep.value -= 1;
}

function goNext() {
  if (isLastStep.value) return;
  transitionDirection.value = "forward";
  activeStep.value += 1;
}

function skipToLastStep() {
  transitionDirection.value = "forward";
  activeStep.value = steps.length - 1;
}
</script>
