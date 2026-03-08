import { en_gb } from "@nuxt/ui/locale";
import { computed } from "vue";
import { resolveAppLocale } from "~/utils/i18n";

const nuxtUiLocales = {
  en_gb
} as const;

export function useAppLocale() {
  const { locale } = useI18n();

  const appLocale = computed(() => resolveAppLocale(locale.value));
  const code = computed(() => appLocale.value.code);
  const lang = computed(() => appLocale.value.language);
  const dir = computed(() => appLocale.value.dir);

  const uiLocale = computed(() => {
    const baseLocale = nuxtUiLocales[appLocale.value.nuxtUiLocale];

    return {
      ...baseLocale,
      code: lang.value,
      dir: dir.value
    };
  });

  return {
    appLocale,
    code,
    dir,
    lang,
    uiLocale
  };
}
