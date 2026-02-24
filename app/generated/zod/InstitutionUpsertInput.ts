// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const institutionUpsertInputGeneratedSchema = z.object({ name: z.string().regex(new RegExp(".*\\S.*")).min(1).max(120) });
export type InstitutionUpsertInputFromSchema = z.infer<typeof institutionUpsertInputGeneratedSchema>;
