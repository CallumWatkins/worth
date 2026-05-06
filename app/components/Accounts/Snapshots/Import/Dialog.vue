<template>
  <UModal
    v-model:open="open"
    title="Import snapshots"
    :dismissible="stepIndex === 0"
    :ui="{ content: 'max-w-3xl' }"
  >
    <template #body>
      <div class="space-y-8">
        <UStepper
          v-if="stepItems.length > 1"
          v-model="stepIndex"
          :items="stepItems"
          disabled
          size="sm"
          class="w-full"
        />

        <UAlert
          v-if="errorMessage"
          color="error"
          variant="subtle"
          :title="errorMessage"
        />

        <div v-if="stepIndex === 0" class="space-y-5">
          <p class="leading-7 text-toned">
            Import balance snapshots from outside Worth into this account.
          </p>

          <h3 class="mb-3 text-sm font-medium text-center">
            Choose source
          </h3>

          <div class="flex flex-wrap justify-center gap-3">
            <button
              v-for="flow in flowDefinitions"
              :key="flow.id"
              type="button"
              class="w-full max-w-xs cursor-pointer rounded-xl border border-default p-4 text-left transition hover:border-primary/60 hover:bg-primary/5 disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="busy"
              @click="selectFlow(flow.id)"
            >
              <div class="flex items-center gap-2 font-semibold text-highlighted">
                <UIcon :name="flow.icon" class="size-5 text-primary" />
                {{ flow.label }}
              </div>
              <div class="mt-2 text-sm text-muted">
                {{ flow.description }}
              </div>
            </button>
          </div>
        </div>

        <component
          :is="currentStep.component"
          v-else-if="currentStep"
          v-bind="currentStep.props()"
        />

        <div
          v-if="(stepIndex > 0) || showContinue || showComplete"
          class="flex items-center justify-between gap-2"
        >
          <div>
            <UButton
              v-if="stepIndex > 0"
              color="neutral"
              variant="subtle"
              :disabled="busy"
              @click="goBack"
            >
              Back
            </UButton>
          </div>

          <div class="flex justify-end gap-2">
            <UButton
              v-if="showContinue"
              :disabled="!canContinue || busy"
              :loading="busy"
              @click="goNext"
            >
              Continue
            </UButton>
            <UButton
              v-else-if="showComplete && isCompleteNoop"
              color="neutral"
              variant="subtle"
              :disabled="!canComplete || busy"
              :loading="busy"
              @click="completeImport"
            >
              Done
            </UButton>
            <UButton
              v-else-if="showComplete"
              :disabled="!canComplete || busy"
              :loading="busy"
              @click="completeImport"
            >
              {{ selectedFlow?.completeLabel ?? 'Import snapshots' }}
            </UButton>
          </div>
        </div>
      </div>
    </template>
  </UModal>
</template>

<script lang="ts" setup>
import type { StepperItem } from "@nuxt/ui";
import type { CurrencyCode } from "~/generated/bindings";
import type { SnapshotImportFlowDefinition } from "~/utils/snapshot-import-flows";
import { stepperItemFromImportStep } from "~/utils/snapshot-import-flows";

const props = defineProps<{
  accountId: number | null
  currencyCode: CurrencyCode
}>();

const open = defineModel<boolean>("open", { required: true });

const stepIndex = ref(0);
const selectedFlowId = ref<string | null>(null);
const errorMessage = ref<string | null>(null);

const accountId = toRef(props, "accountId");
const currencyCode = toRef(props, "currencyCode");

const flowDefinitions: SnapshotImportFlowDefinition[] = [
  useCsvSnapshotImportFlow({
    accountId,
    currencyCode,
    setErrorMessage: (message) => {
      errorMessage.value = message;
    },
    onComplete: () => {
      open.value = false;
    }
  })
];

const selectedFlow = computed(() => {
  if (selectedFlowId.value == null) return null;
  return flowDefinitions.find((flow) => flow.id === selectedFlowId.value) ?? null;
});

const flowSteps = computed(() => selectedFlow.value?.steps() ?? []);

const currentStep = computed(() => {
  if (stepIndex.value === 0) return null;
  return flowSteps.value[stepIndex.value - 1] ?? null;
});

const stepItems = computed<StepperItem[]>(() => [
  { title: "Source", icon: "i-lucide-database" },
  ...flowSteps.value.map(stepperItemFromImportStep)
]);

const busy = computed(() => selectedFlow.value?.isBusy() ?? false);
const canContinue = computed(() => currentStep.value?.canContinue() ?? false);
const canComplete = computed(() => selectedFlow.value?.canComplete() ?? false);
const isCompleteNoop = computed(() => selectedFlow.value?.isCompleteNoop?.() ?? false);
const showContinue = computed(() => stepIndex.value > 0 && stepIndex.value < flowSteps.value.length);
const showComplete = computed(() => stepIndex.value > 0 && stepIndex.value === flowSteps.value.length);

watch(open, (isOpen) => {
  if (!isOpen) return;
  reset();
});

function selectFlow(flowId: string) {
  const flow = flowDefinitions.find((candidate) => candidate.id === flowId);
  if (!flow) return;

  errorMessage.value = null;
  flow.reset();
  selectedFlowId.value = flow.id;
  stepIndex.value = 1;
}

function goBack() {
  errorMessage.value = null;
  currentStep.value?.onBack?.();

  if (stepIndex.value === 1) {
    selectedFlowId.value = null;
    stepIndex.value = 0;
    return;
  }

  stepIndex.value -= 1;
}

async function goNext() {
  const step = currentStep.value;
  if (!step) return;

  errorMessage.value = null;
  const canMove = await (step.beforeNext?.() ?? true);
  if (!canMove) return;

  stepIndex.value += 1;
}

async function completeImport() {
  await selectedFlow.value?.complete();
}

function reset() {
  stepIndex.value = 0;
  selectedFlowId.value = null;
  errorMessage.value = null;
  for (const flow of flowDefinitions) {
    flow.reset();
  }
}
</script>
