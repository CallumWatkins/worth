import type {
  AccountSnapshotsCreateInput,
  AccountSnapshotsDeleteInput,
  AccountSnapshotUpdateInput,
  SnapshotImportOptionsInput,
  SnapshotImportSourceInput
} from "~/generated/bindings";

import { useMutation, useQueryClient } from "@tanstack/vue-query";

export const useAccountSnapshotMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();

  const invalidateSnapshotWrites = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() })
    ]);
  };

  const createSnapshots = proxyRefs(useMutation({
    mutationFn: async ({ accountId, input }: { accountId: number, input: AccountSnapshotsCreateInput }) =>
      api.accountSnapshotsCreate(accountId, input),
    onSuccess: invalidateSnapshotWrites
  }));

  const updateSnapshot = proxyRefs(useMutation({
    mutationFn: async ({
      accountId,
      snapshotId,
      input
    }: {
      accountId: number
      snapshotId: number
      input: AccountSnapshotUpdateInput
    }) => api.accountSnapshotUpdate(accountId, snapshotId, input),
    onSuccess: invalidateSnapshotWrites
  }));

  const deleteSnapshots = proxyRefs(useMutation({
    mutationFn: async ({ accountId, input }: { accountId: number, input: AccountSnapshotsDeleteInput }) =>
      api.accountSnapshotsDelete(accountId, input),
    onSuccess: invalidateSnapshotWrites
  }));

  const importSnapshots = proxyRefs(useMutation({
    mutationFn: async ({
      accountId,
      input,
      options
    }: {
      accountId: number
      input: SnapshotImportSourceInput
      options: SnapshotImportOptionsInput
    }) => api.accountSnapshotImportCommit(accountId, input, options),
    onSuccess: invalidateSnapshotWrites
  }));

  return {
    createSnapshots,
    updateSnapshot,
    deleteSnapshots,
    importSnapshots
  };
};
