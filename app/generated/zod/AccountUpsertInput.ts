// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const accountUpsertInputGeneratedSchema = z.object({ account_type: z.enum(["current", "savings", "credit_card", "isa", "investment", "pension", "cash", "loan"]), currency_code: z.string().regex(new RegExp("^[A-Z]{3}$")).min(3).max(3), institution: z.discriminatedUnion("kind", [z.object({ id: z.number().int().gte(1), kind: z.literal("existing") }), z.object({ input: z.object({ name: z.string().regex(new RegExp(".*\\S.*")).min(1).max(120) }), kind: z.literal("new") })]), name: z.string().regex(new RegExp(".*\\S.*")).min(1).max(120), normal_balance_sign: z.union([z.literal(-1), z.literal(1)]), opened_date: z.preprocess((value) => value === "" ? null : value, z.union([z.string().date(), z.null()]).default(null)).optional() });
export type AccountUpsertInputFromSchema = z.infer<typeof accountUpsertInputGeneratedSchema>;
