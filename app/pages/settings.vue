<template>
  <UContainer>
    <UPageHeader
      title="Settings"
      description="Manage app preferences"
      :ui="{
        root: 'pb-0 border-none',
        description: 'mt-1'
      }"
    />

    <UPageBody class="space-y-6 max-w-3xl">
      <UAlert
        v-if="settingsQuery.isError"
        color="error"
        variant="subtle"
        orientation="horizontal"
        :title="settingsQuery.error.message"
        :actions="hasErrorDetailsSurvey ? [getErrorDetailsSurveyAction()] : []"
      />

      <UAlert
        v-if="saveError"
        color="error"
        variant="subtle"
        orientation="horizontal"
        :title="saveError"
        :actions="hasErrorDetailsSurvey ? [getErrorDetailsSurveyAction()] : []"
      />

      <UPageCard
        title="General"
      >
        <div class="space-y-6">
          <UFormField
            label="Share anonymous diagnostics and feedback"
            description="Helps improve Worth by sharing anonymous usage events, crash reports, and any feedback you choose to send. Personal or financial details are never sent."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <USwitch
              :model-value="analyticsEnabled"
              :disabled="isSettingsBusy"
              :loading="pendingField === 'analytics'"
              aria-label="Share anonymous usage and crash data"
              @update:model-value="onAnalyticsEnabledUpdate"
            />
          </UFormField>
        </div>
      </UPageCard>

      <UPageCard
        title="Display"
      >
        <div class="space-y-6">
          <UFormField
            label="Default display currency"
            description="Aggregated balances use this currency, using conversion if necessary."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <USelect
              :model-value="defaultDisplayCurrencyCode"
              :items="currencyItems"
              class="min-w-25"
              :ui="{ content: 'min-w-fit' }"
              :content="{ align: 'end' }"
              :disabled="isSettingsBusy"
              :loading="pendingField === 'currency'"
              @update:model-value="onDefaultDisplayCurrencyCodeUpdate"
            />
          </UFormField>

          <UFormField
            label="Display locale"
            description="Controls date, number, and language formatting."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <USelect
              :model-value="displayLocale"
              :items="localeItems"
              class="min-w-25"
              :ui="{ content: 'min-w-fit' }"
              :content="{ align: 'end' }"
              :disabled="isSettingsBusy"
              :loading="pendingField === 'locale'"
              @update:model-value="onDisplayLocaleUpdate"
            />
          </UFormField>

          <UFormField
            label="Theme"
            description="Choose how Worth looks."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <USelect
              :model-value="theme"
              :items="themeItems"
              class="min-w-25"
              :ui="{ content: 'min-w-fit' }"
              :content="{ align: 'end' }"
              :disabled="isSettingsBusy"
              :loading="pendingField === 'theme'"
              @update:model-value="onThemeUpdate"
            />
          </UFormField>
        </div>
      </UPageCard>

      <UPageCard title="About">
        <div class="space-y-6">
          <p class="text-sm text-muted">
            Worth is made in the open by <ULink
              to="https://www.callumwatkins.com"
              external
              @click.prevent="openUrl('https://www.callumwatkins.com')"
            >
              Callum Watkins
            </ULink> in London as a cleaner home for long-term balance tracking. You can view the source code, report issues, give feedback, or say thanks from here.
          </p>

          <UFormField
            label="Open source"
            description="View the source code, report issues, and contribute on GitHub."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <UButton
              icon="i-lucide-github"
              class="whitespace-nowrap"
              variant="subtle"
              @click="openUrl('https://github.com/CallumWatkins/worth')"
            >
              View on GitHub
            </UButton>
          </UFormField>

          <UFormField
            v-if="hasFeedbackSurvey"
            label="Feedback"
            description="Share a bug, suggestion, or piece of feedback."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <UButton
              icon="i-lucide-message-circle-heart"
              class="whitespace-nowrap"
              variant="subtle"
              @click="openFeedbackSurvey"
            >
              Give feedback
            </UButton>
          </UFormField>

          <UFormField
            label="Support Worth"
            description="Worth is free to use. If it has been useful to you and you would like to say thanks, a small donation means a lot."
            orientation="horizontal"
            class="items-center gap-25"
          >
            <UButton
              icon="i-lucide-heart"
              class="whitespace-nowrap"
              variant="subtle"
              @click="openUrl('https://ko-fi.com/callumwatkins')"
            >
              Donate on Ko-fi
            </UButton>
          </UFormField>
        </div>
      </UPageCard>
    </UPageBody>
  </UContainer>
</template>

<script setup lang="ts">
import type { AnalyticsEventProperties } from "~/composables/useAnalytics";
import type { AppLocaleCode, AppSettingsDto, AppSettingsUpdateInput, CurrencyCode, ThemePreference } from "~/generated/bindings";

import { openUrl } from "@tauri-apps/plugin-opener";
import { supportedCurrencyCodes } from "~/utils/currencies";
import { APP_LOCALES } from "~/utils/i18n";

type SettingsField = "analytics" | "currency" | "locale" | "theme";

const {
  settings,
  settingsQuery,
  updateSettings,
  updateSetting
} = useSettingsManager();

const saveError = ref<string | null>(null);
const pendingField = ref<SettingsField | null>(null);
const analyticsEnabled = ref(true);
const defaultDisplayCurrencyCode = ref<CurrencyCode>();
const displayLocale = ref<AppLocaleCode>();
const theme = ref<ThemePreference>();
const isSettingsBusy = computed(() => settingsQuery.isPending || unref(updateSettings.isPending));
const { hasFeedbackSurvey, openFeedbackSurvey } = useFeedbackSurvey();
const { hasErrorDetailsSurvey, getErrorDetailsSurveyAction } = useErrorDetailsSurvey();
const { captureAnalyticsEvent } = useAnalytics();

const currencyItems = supportedCurrencyCodes.map((currencyCode) => ({
  label: currencyCode,
  value: currencyCode
}));

const localeItems = [
  { label: "System default", value: "system" },
  ...APP_LOCALES.map((locale) => ({
    label: locale.name,
    value: locale.code
  }))
] satisfies { label: string, value: AppLocaleCode }[];

const themeItems = [...themePreferenceItems];

function syncEditableSettings(value: AppSettingsDto) {
  analyticsEnabled.value = value.analytics_enabled;
  defaultDisplayCurrencyCode.value = value.default_display_currency_code;
  displayLocale.value = value.display_locale;
  theme.value = value.theme;
}

watch(settings, syncEditableSettings, { immediate: true });

async function saveSetting(field: SettingsField, patch: Partial<AppSettingsUpdateInput>) {
  const previous = settings.value;
  const startedAt = performance.now();
  const analyticsProperties = getSettingAnalyticsProperties(field, patch);

  saveError.value = null;
  pendingField.value = field;

  try {
    await updateSetting(patch);
    if (analyticsProperties) {
      captureAnalyticsEvent("settings:setting_update", analyticsProperties, {
        operationStartedAt: startedAt
      });
    }
  } catch (error) {
    if (analyticsProperties) {
      captureAnalyticsEvent("settings:setting_update_fail", {
        ...analyticsProperties,
        ...getAnalyticsErrorProperties(error)
      }, {
        operationStartedAt: startedAt
      });
    }

    syncEditableSettings(previous);
    saveError.value = error instanceof Error ? error.message : "Failed to save settings";
  } finally {
    pendingField.value = null;
  }
}

function getSettingAnalyticsProperties(field: SettingsField, patch: Partial<AppSettingsUpdateInput>): AnalyticsEventProperties | null {
  switch (field) {
  case "analytics":
    return null;
  case "currency":
    if (patch.default_display_currency_code == null) return null;
    return {
      setting_name: "default_display_currency_code",
      setting_value: patch.default_display_currency_code,
      $set: { desktop_app_default_display_currency_code: patch.default_display_currency_code }
    };
  case "locale":
    if (patch.display_locale == null) return null;
    return {
      setting_name: "display_locale",
      setting_value: patch.display_locale,
      $set: { desktop_app_display_locale: patch.display_locale }
    };
  case "theme":
    if (patch.theme == null) return null;
    return {
      setting_name: "theme",
      setting_value: patch.theme,
      $set: { desktop_app_theme: patch.theme }
    };
  }
}

function onAnalyticsEnabledUpdate(analytics_enabled: boolean) {
  if (analytics_enabled === settings.value.analytics_enabled) return;
  analyticsEnabled.value = analytics_enabled;
  void saveSetting("analytics", { analytics_enabled });
}

function onDefaultDisplayCurrencyCodeUpdate(default_display_currency_code: CurrencyCode) {
  if (default_display_currency_code === settings.value.default_display_currency_code) return;
  defaultDisplayCurrencyCode.value = default_display_currency_code;
  void saveSetting("currency", { default_display_currency_code });
}

function onDisplayLocaleUpdate(display_locale: AppLocaleCode) {
  if (display_locale === settings.value.display_locale) return;
  displayLocale.value = display_locale;
  void saveSetting("locale", { display_locale });
}

function onThemeUpdate(nextTheme: ThemePreference) {
  if (nextTheme === settings.value.theme) return;
  theme.value = nextTheme;
  void saveSetting("theme", { theme: nextTheme });
}
</script>
