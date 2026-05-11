// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const accountClassificationGeneratedSchema = z.enum(["asset","liability"]);
export type AccountClassificationFromSchema = z.infer<typeof accountClassificationGeneratedSchema>;
