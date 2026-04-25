// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const institutionUpsertInputGeneratedSchema = z.object({ "name": z.string({ error: "Enter an institution name" }).regex(new RegExp(".*\\S.*"), { error: "Enter an institution name" }).min(1, { error: "Enter an institution name" }).max(120, { error: "Institution name must be 120 characters or fewer" }) });
export type InstitutionUpsertInputFromSchema = z.infer<typeof institutionUpsertInputGeneratedSchema>;
