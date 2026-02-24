import type { z } from "zod";

import {
  accountUpsertInputGeneratedSchema,
  institutionUpsertInputGeneratedSchema
} from "~/generated/zod";

export const accountFormSchema = accountUpsertInputGeneratedSchema;
export const institutionFormSchema = institutionUpsertInputGeneratedSchema;

export type AccountFormInputValues = z.input<typeof accountFormSchema>;
export type AccountFormValues = z.output<typeof accountFormSchema>;
export type InstitutionFormInputValues = z.input<typeof institutionFormSchema>;
export type InstitutionFormValues = z.output<typeof institutionFormSchema>;
