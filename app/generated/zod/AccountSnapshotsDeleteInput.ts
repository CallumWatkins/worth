// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const accountSnapshotsDeleteInputGeneratedSchema = z.object({ "snapshot_ids": z.array(z.number().int(), { error: "Select at least one snapshot" }).min(1, { error: "Select at least one snapshot" }) });
export type AccountSnapshotsDeleteInputFromSchema = z.infer<typeof accountSnapshotsDeleteInputGeneratedSchema>;
