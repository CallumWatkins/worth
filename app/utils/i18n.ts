export const APP_LOCALES = [
  {
    code: "en-GB",
    language: "en-GB",
    name: "English (United Kingdom)",
    dir: "ltr",
    nuxtUiLocale: "en_gb"
  }
] as const;

export type AppLocaleDefinition = (typeof APP_LOCALES)[number];
export type AppLocaleCode = AppLocaleDefinition["code"];

export const DEFAULT_LOCALE: AppLocaleCode = APP_LOCALES[0].code;

export const NUXT_I18N_LOCALES = APP_LOCALES.map(({ code, language, name, dir }) => ({
  code,
  language,
  name,
  dir
}));

export const APP_I18N_CONFIG = {
  defaultLocale: DEFAULT_LOCALE,
  locales: NUXT_I18N_LOCALES,
  strategy: "no_prefix"
} as const;

export function resolveAppLocale(code: string | null | undefined): AppLocaleDefinition {
  return APP_LOCALES.find((locale) => locale.code === code) ?? APP_LOCALES[0];
}
