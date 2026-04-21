// Generated file, update with `bun run contracts:gen`.
import { z } from "zod";

export const accountSnapshotsDeleteInputGeneratedSchema = z.object({ "snapshot_ids": z.array(z.number().int()).min(1) });
export type AccountSnapshotsDeleteInputFromSchema = z.infer<typeof accountSnapshotsDeleteInputGeneratedSchema>;
