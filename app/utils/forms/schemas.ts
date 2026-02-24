import { z } from "zod";

import {
  accountUpsertInputGeneratedSchema,
  institutionUpsertInputGeneratedSchema
} from "~/generated/zod";

interface DateLike {
  toString: () => string
}

function isDateLike(value: unknown): value is DateLike {
  if (value == null || typeof value !== "object") return false;
  const candidate = value as { toString?: unknown };
  return typeof candidate.toString === "function";
}

const openedDateUiSchema = z
  .custom<DateLike | null | undefined>((value) => value == null || isDateLike(value))
  .optional()
  .transform((value) => value?.toString() ?? null);

export const accountFormSchema = accountUpsertInputGeneratedSchema.extend({
  opened_date: openedDateUiSchema
});

export const institutionFormSchema = institutionUpsertInputGeneratedSchema;

export type AccountFormInputValues = z.input<typeof accountFormSchema>;
export type AccountFormValues = z.output<typeof accountFormSchema>;
export type InstitutionFormInputValues = z.input<typeof institutionFormSchema>;
export type InstitutionFormValues = z.output<typeof institutionFormSchema>;
