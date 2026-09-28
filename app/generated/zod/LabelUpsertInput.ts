// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const labelUpsertInputGeneratedSchema = z.object({ "description": z.union([z.string().refine((value) => { let length = 0; for (const _character of value) { length += 1; if (length > 250) return false; } return true; }, { error: "Label description must be 250 characters or fewer" }), z.null()]).optional(), "name": z.string({ error: "Enter a label name" }).regex(new RegExp("^[^\\u0000]*[^\\s\\u0000][^\\u0000]*$"), { error: "Enter a label name" }).refine((value) => { let length = 0; for (const _character of value) { length += 1; if (length >= 1) return true; } return false; }, { error: "Enter a label name" }).refine((value) => { let length = 0; for (const _character of value) { length += 1; if (length > 20) return false; } return true; }, { error: "Label name must be 20 characters or fewer" }) });
export type LabelUpsertInputFromSchema = z.infer<typeof labelUpsertInputGeneratedSchema>;
