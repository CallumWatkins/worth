<template>
  <UModal
    title="Feedback"
    :ui="{ content: 'max-w-xl' }"
    :dismissible="!hasCompletedField"
    :close="{ onClick: closeSurvey }"
    @update:open="handleOpenUpdate"
  >
    <template #body>
      <p class="text-sm text-muted">
        Your feedback helps me understand what is useful, confusing, broken, or missing, and is greatly appreciated. Please avoid including any personal financial details.
      </p>
      <UForm
        id="feedback-survey-form"
        :state="surveyState"
        class="mt-6 space-y-4"
        @submit="submitSurvey"
      >
        <UFormField label="Rate Worth" hint="Optional">
          <StarRating
            v-model="rating"
            :max="5"
          />
        </UFormField>

        <UFormField label="Share your thoughts" hint="Optional">
          <UTextarea
            v-model="feedback"
            autoresize
            :rows="4"
            :maxrows="8"
            class="w-full"
          />
        </UFormField>

        <UFormField label="Email" description="If you would like to receive a response." hint="Optional">
          <UInput
            v-model="email"
            type="email"
            autocomplete="email"
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
          form="feedback-survey-form"
          :disabled="!hasCompletedField"
        >
          Submit
        </UButton>
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import type { Properties } from "posthog-js";

const emit = defineEmits<{
  close: []
}>();

const posthog = usePostHog();
const toast = useToast();

const closed = ref(false);
const rating = ref<number | null>(null);
const feedback = ref("");
const email = ref("");

const surveyState = computed(() => ({
  rating: rating.value,
  feedback: feedback.value,
  email: email.value
}));
const trimmedFeedback = computed(() => feedback.value.trim());
const trimmedEmail = computed(() => email.value.trim());
const hasCompletedField = computed(() => rating.value != null || trimmedFeedback.value !== "" || trimmedEmail.value !== "");

function handleOpenUpdate(open: boolean) {
  if (!open) closeSurvey();
}

function closeSurvey() {
  if (closed.value) return;

  posthog?.capture(hasCompletedField.value ? "survey abandoned" : "survey dismissed", { $survey_id: "019dfe8e-ec1b-0000-5f3f-49ca32eae32b" });
  closeDialog();
}

function submitSurvey() {
  if (!hasCompletedField.value) {
    closeSurvey();
    return;
  }

  const properties: Properties = { $survey_id: "019dfe8e-ec1b-0000-5f3f-49ca32eae32b" };

  if (rating.value != null) {
    properties["$survey_response_efb7a2a0-2235-4155-8f79-c38c3322243e"] = String(rating.value);
  }

  if (trimmedFeedback.value !== "") {
    properties["$survey_response_f4c4bc96-2b56-4054-a26c-443aab736600"] = trimmedFeedback.value;
  }

  if (trimmedEmail.value !== "") {
    properties["$survey_response_ffb522cd-0a72-423a-b00f-5da7786f677e"] = trimmedEmail.value;
  }

  posthog?.capture("survey sent", properties);
  closeDialog();

  toast.add({
    title: "Feedback sent",
    description: "Thanks for helping improve Worth.",
    color: "success"
  });
}

function closeDialog() {
  closed.value = true;
  resetSurveyForm();
  emit("close");
}

function resetSurveyForm() {
  rating.value = null;
  feedback.value = "";
  email.value = "";
}
</script>
