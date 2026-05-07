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
        :title="settingsQuery.error.message"
      />

      <UAlert
        v-if="saveError"
        color="error"
        variant="subtle"
        :title="saveError"
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

      <UPageCard
        v-if="hasFeedbackSurvey"
        title="Feedback"
        description="Help shape the future of Worth."
        :ui="{
          container: 'lg:flex flex-row gap-25 items-center'
        }"
      >
        <UButton
          icon="i-lucide-message-circle-heart"
          @click="openFeedbackSurvey"
        >
          Give feedback
        </UButton>
      </UPageCard>
    </UPageBody>
  </UContainer>
</template>

<script setup lang="ts">
import type { AppLocaleCode, AppSettingsDto, AppSettingsUpdateInput, CurrencyCode, ThemePreference } from "~/generated/bindings";
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

  saveError.value = null;
  pendingField.value = field;

  try {
    await updateSetting(patch);
  } catch (error) {
    syncEditableSettings(previous);
    saveError.value = error instanceof Error ? error.message : "Failed to save settings";
  } finally {
    pendingField.value = null;
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
