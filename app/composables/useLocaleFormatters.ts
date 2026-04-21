import type { CurrencyCode } from "~/generated/bindings";

const numberFormatterCache = new Map<string, Intl.NumberFormat>();
const dateFormatterCache = new Map<string, Intl.DateTimeFormat>();

function getNumberFormatter(locale: string, options: Intl.NumberFormatOptions) {
  const key = `${locale}:${JSON.stringify(options)}`;
  const existing = numberFormatterCache.get(key);

  if (existing) {
    return existing;
  }

  const formatter = new Intl.NumberFormat(locale, options);
  numberFormatterCache.set(key, formatter);
  return formatter;
}

function getDateFormatter(locale: string, options: Intl.DateTimeFormatOptions) {
  const key = `${locale}:${JSON.stringify(options)}`;
  const existing = dateFormatterCache.get(key);

  if (existing) {
    return existing;
  }

  const formatter = new Intl.DateTimeFormat(locale, options);
  dateFormatterCache.set(key, formatter);
  return formatter;
}

export function useLocaleFormatters() {
  const { code } = useAppLocale();

  function formatCurrency(value: number, currencyCode: CurrencyCode, options: Intl.NumberFormatOptions = {}) {
    return getNumberFormatter(code.value, {
      style: "currency",
      currency: currencyCode,
      ...options
    }).format(value);
  }

  function formatCurrencyMinor(minor: number, currencyCode: CurrencyCode, options: Intl.NumberFormatOptions = {}) {
    return formatCurrency(convertCurrencyMinorUnitsToMajorAmount(minor), currencyCode, options);
  }

  function formatDate(iso: string | null | undefined, options: Intl.DateTimeFormatOptions, fallback = "—") {
    const date = getDateObjectFromCalendarDateIsoString(iso);

    if (date == null || Number.isNaN(date.getTime())) {
      return fallback;
    }

    return getDateFormatter(code.value, options).format(date);
  }

  function formatShortDate(iso: string | null | undefined, fallback = "—") {
    return formatDate(iso, {
      day: "2-digit",
      month: "short",
      year: "numeric"
    }, fallback);
  }

  return {
    formatCurrency,
    formatCurrencyMinor,
    formatDate,
    formatShortDate
  };
}
