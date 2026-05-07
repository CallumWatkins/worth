import { LazyFeedbackSurveyDialog } from "#components";

export function useFeedbackSurvey() {
  const settings = useSettings();
  const posthog = usePostHog();
  const overlay = useOverlay();
  const isSurveyAvailable = ref(false);
  const dialogOpen = useState("feedbackSurveyDialogOpen", () => false);

  const hasFeedbackSurvey = computed(() => settings.value.analytics_enabled && isSurveyAvailable.value);

  if (posthog) {
    const unsubscribeSurveys = posthog.onSurveysLoaded((surveys) => {
      isSurveyAvailable.value = surveys.some((survey) => survey.id === "019dfe8e-ec1b-0000-5f3f-49ca32eae32b" && survey.end_date === null);
    });

    onScopeDispose(unsubscribeSurveys);
  }

  async function openFeedbackSurvey() {
    if (!hasFeedbackSurvey.value || dialogOpen.value) return;

    posthog?.capture("survey shown", { $survey_id: "019dfe8e-ec1b-0000-5f3f-49ca32eae32b" });
    dialogOpen.value = true;

    try {
      await overlay.create(LazyFeedbackSurveyDialog, { destroyOnClose: true }).open().result;
    } finally {
      dialogOpen.value = false;
    }
  }

  return {
    hasFeedbackSurvey,
    openFeedbackSurvey
  };
}
