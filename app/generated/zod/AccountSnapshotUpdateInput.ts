// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const accountSnapshotUpdateInputGeneratedSchema = z.object({ "balance_minor": z.number().int(), "date": z.custom<import("@internationalized/date").CalendarDate>((value) => value != null && typeof value === "object" && typeof value.toString === "function").transform((value) => value.toString()).pipe(z.string().date()), "overwrite_existing": z.boolean() });
export type AccountSnapshotUpdateInputFromSchema = z.infer<typeof accountSnapshotUpdateInputGeneratedSchema>;
