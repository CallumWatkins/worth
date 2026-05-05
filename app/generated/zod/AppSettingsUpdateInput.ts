// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const appSettingsUpdateInputGeneratedSchema = z.object({ "analytics_enabled": z.union([z.boolean(), z.null()]).optional(), "default_display_currency_code": z.union([z.literal("GBP"), z.null()]).optional(), "display_locale": z.union([z.enum(["system","en-GB"]), z.null()]).optional(), "theme": z.union([z.enum(["system","light","dark"]), z.null()]).optional() });
export type AppSettingsUpdateInputFromSchema = z.infer<typeof appSettingsUpdateInputGeneratedSchema>;
