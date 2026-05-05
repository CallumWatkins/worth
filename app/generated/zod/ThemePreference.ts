// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const themePreferenceGeneratedSchema = z.enum(["system","light","dark"]);
export type ThemePreferenceFromSchema = z.infer<typeof themePreferenceGeneratedSchema>;
