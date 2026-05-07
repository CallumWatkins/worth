import type { ButtonProps } from "@nuxt/ui";
import { LazyErrorDetailsSurveyDialog } from "#components";

export function useErrorDetailsSurvey() {
  const settings = useNullableSettings();
  const posthog = usePostHog();
  const overlay = useOverlay();
  const isSurveyAvailable = useState("errorDetailsSurveyAvailable", () => false);
  const dialogOpen = useState("errorDetailsSurveyDialogOpen", () => false);

  const hasErrorDetailsSurvey = computed(() => settings.value?.analytics_enabled === true && isSurveyAvailable.value);

  if (posthog) {
    const unsubscribeSurveys = posthog.onSurveysLoaded((surveys) => {
      isSurveyAvailable.value = surveys.some((survey) => survey.id === "019e0002-7368-0000-21c0-195e58efc1cc" && survey.end_date === null);
    });

    onScopeDispose(unsubscribeSurveys);
  }

  async function openErrorDetailsSurvey() {
    if (!hasErrorDetailsSurvey.value || dialogOpen.value) return;

    posthog?.capture("survey shown", { $survey_id: "019e0002-7368-0000-21c0-195e58efc1cc" });
    dialogOpen.value = true;

    try {
      await overlay.create(LazyErrorDetailsSurveyDialog, { destroyOnClose: true }).open().result;
    } finally {
      dialogOpen.value = false;
    }
  }

  function getErrorDetailsSurveyAction(overrides: Partial<ButtonProps> = {}): ButtonProps {
    return {
      label: "Send details",
      icon: "i-lucide-message-circle-warning",
      color: "error",
      variant: "ghost",
      ...overrides,
      onClick: async () => {
        await openErrorDetailsSurvey();
      }
    };
  }

  return {
    hasErrorDetailsSurvey,
    getErrorDetailsSurveyAction,
    openErrorDetailsSurvey
  };
}
