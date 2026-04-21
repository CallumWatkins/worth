import type { CurrencyCode } from "~/generated/bindings";
import { currencyCodeGeneratedSchema } from "~/generated/zod";

function supportedCurrencyValuesFromSchema(): CurrencyCode[] {
  const enumSchema = currencyCodeGeneratedSchema as Partial<{ options: readonly CurrencyCode[] }>;
  if (Array.isArray(enumSchema.options))
    return enumSchema.options as CurrencyCode[];

  const literalSchema = currencyCodeGeneratedSchema as Partial<{ value: CurrencyCode }>;
  if (typeof literalSchema.value === "string")
    return [literalSchema.value];

  return [];
}

export const supportedCurrencyCodes = supportedCurrencyValuesFromSchema();

export function convertCurrencyMajorAmountToMinorUnits(amount: number | undefined) {
  if (amount == null || !Number.isFinite(amount)) return null;
  return Math.round(amount * 100);
}

export function convertCurrencyMinorUnitsToMajorAmount(minor: number) {
  return minor / 100;
}

export function normalizeCurrencyMajorAmount(amount: number) {
  return Math.round(amount * 100) / 100;
}

export function normalizeOptionalCurrencyMajorAmount(amount: number | null | undefined) {
  return amount == null ? undefined : normalizeCurrencyMajorAmount(amount);
}

export function parseCurrencyInputNumberEventValue(event: Event) {
  const target = event.target;
  if (!(target instanceof HTMLInputElement)) return undefined;

  const normalized = target.value.replaceAll(/[^\d+\-.]/g, "").trim();
  if (!normalized || normalized === "+" || normalized === "-" || normalized === ".") {
    return undefined;
  }

  const parsed = Number(normalized);
  if (!Number.isFinite(parsed)) return undefined;
  return normalizeCurrencyMajorAmount(parsed);
}
