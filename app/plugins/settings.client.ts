import type { Ref } from "vue";
import type { AppSettingsDto } from "~/generated/bindings";
import { useQueryClient } from "@tanstack/vue-query";

export default defineNuxtPlugin({
  name: "settings",
  dependsOn: ["vue-query", "posthog-super-properties", "i18n:plugin:route-locale-detect"],
  async setup(nuxtApp) {
    const api = useApi();
    const queryClient = useQueryClient();
    const settings = useNullableSettings();
    const colorMode = useColorMode();
    const i18n = nuxtApp.$i18n as { locale: Ref<string> };
    const systemDisplayLocale = i18n.locale.value;

    settings.value = await api.settingsGet();
    queryClient.setQueryData(queryKeys.settings.get(), settings.value);

    function applySettings(value: AppSettingsDto | null) {
      if (value == null) return;

      const displayLocale = value.display_locale === "system" ? systemDisplayLocale : value.display_locale;

      if (i18n.locale.value !== displayLocale) {
        i18n.locale.value = displayLocale;
      }

      if (colorMode.preference !== value.theme) {
        colorMode.preference = value.theme;
      }

      const posthog = usePostHog();
      if (!posthog) return;

      const posthogOptedOut = posthog.has_opted_out_capturing();

      if (!value.analytics_enabled && !posthogOptedOut) {
        posthog.opt_out_capturing();
      } else if (value.analytics_enabled && posthogOptedOut) {
        posthog.opt_in_capturing();
      }
    }

    watch(settings, applySettings, { immediate: true, deep: true });
    nuxtApp.hook("app:mounted", () => applySettings(settings.value));
  }
});
