import type { CurrencyCode } from "~/generated/bindings";
import { currencyCodeGeneratedSchema } from "~/generated/zod";

function supportedCurrencyValuesFromSchema(): CurrencyCode[] {
  const enumSchema = currencyCodeGeneratedSchema as Partial<{ options: readonly CurrencyCode[] }>;
  if (Array.isArray(enumSchema.options))
    return [...enumSchema.options];

  const literalSchema = currencyCodeGeneratedSchema as Partial<{ value: CurrencyCode }>;
  if (typeof literalSchema.value === "string")
    return [literalSchema.value];

  return [];
}

export const supportedCurrencyCodes = supportedCurrencyValuesFromSchema();
