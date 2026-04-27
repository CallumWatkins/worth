import type { StepperItem } from "@nuxt/ui";
import type { Component } from "vue";

export interface SnapshotImportFlowStep {
  id: string
  title: string
  icon: string
  component: Component
  props: () => Record<string, unknown>
  canContinue: () => boolean
  beforeNext?: () => Promise<boolean> | boolean
  onBack?: () => void
}

export interface SnapshotImportFlowDefinition {
  id: string
  label: string
  description: string
  icon: string
  steps: () => SnapshotImportFlowStep[]
  isBusy: () => boolean
  reset: () => void
  canComplete: () => boolean
  complete: () => Promise<boolean>
  completeLabel: string
}

export function stepperItemFromImportStep(step: SnapshotImportFlowStep): StepperItem {
  return {
    title: step.title,
    icon: step.icon
  };
}
