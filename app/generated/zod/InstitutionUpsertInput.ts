// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const institutionUpsertInputGeneratedSchema = z.object({ "name": z.string({ error: "Enter an institution name" }).regex(new RegExp(".*\\S.*"), { error: "Enter an institution name" }).refine((value) => { let length = 0; for (const _character of value) { length += 1; if (length >= 1) return true; } return false; }, { error: "Enter an institution name" }).refine((value) => { let length = 0; for (const _character of value) { length += 1; if (length > 80) return false; } return true; }, { error: "Institution name must be 80 characters or fewer" }) });
export type InstitutionUpsertInputFromSchema = z.infer<typeof institutionUpsertInputGeneratedSchema>;
