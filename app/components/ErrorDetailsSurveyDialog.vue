<template>
  <UModal
    title="Send error details"
    :ui="{ content: 'max-w-xl' }"
    :dismissible="!hasCompletedField"
    :close="{ onClick: closeSurvey }"
    @update:open="handleOpenUpdate"
  >
    <template #body>
      <p class="text-sm text-toned">
        A short description of what you were trying to do when you encountered this problem can make it easier to fix. Please avoid including any personal financial details.
      </p>
      <UForm
        id="error-details-survey-form"
        :state="surveyState"
        class="mt-6 space-y-4"
        @submit="submitSurvey"
      >
        <UFormField label="Describe what happened">
          <UTextarea
            v-model="details"
            autoresize
            autofocus
            :rows="5"
            :maxrows="10"
            class="w-full"
          />
        </UFormField>
      </UForm>
    </template>

    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton
          type="button"
          color="neutral"
          variant="ghost"
          @click="closeSurvey"
        >
          Cancel
        </UButton>
        <UButton
          type="submit"
          form="error-details-survey-form"
          :disabled="!hasCompletedField"
        >
          Submit
        </UButton>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
const emit = defineEmits<{
  close: []
}>();

const posthog = usePostHog();
const toast = useToast();

const closed = ref(false);
const details = ref("");

const surveyState = computed(() => ({ details: details.value }));
const trimmedDetails = computed(() => details.value.trim());
const hasCompletedField = computed(() => trimmedDetails.value !== "");

function handleOpenUpdate(open: boolean) {
  if (!open) closeSurvey();
}

function closeSurvey() {
  if (closed.value) return;

  posthog?.capture(hasCompletedField.value ? "survey abandoned" : "survey dismissed", {
    $survey_id: "019e0002-7368-0000-21c0-195e58efc1cc"
  });
  closeDialog();
}

function submitSurvey() {
  if (!hasCompletedField.value) {
    closeSurvey();
    return;
  }

  posthog?.capture("survey sent", {
    $survey_id: "019e0002-7368-0000-21c0-195e58efc1cc",
    "$survey_response_578793b1-f402-45ac-b090-42061120eeee": trimmedDetails.value
  });
  closeDialog();

  toast.add({
    title: "Error details sent",
    description: "Thanks for helping improve Worth.",
    color: "success"
  });
}

function closeDialog() {
  closed.value = true;
  details.value = "";
  emit("close");
}
</script>
