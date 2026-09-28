import type { LabelUpsertInput } from "~/generated/bindings";
import { useMutation, useQueryClient } from "@tanstack/vue-query";

export const useLabelMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();

  const invalidateLabelWrites = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.labels.list() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() })
    ]);
  };

  const createLabel = proxyRefs(useMutation({
    mutationFn: async (input: LabelUpsertInput) => api.labelsCreate(input),
    onSuccess: invalidateLabelWrites
  }));
  const updateLabel = proxyRefs(useMutation({
    mutationFn: async ({ labelId, input }: { labelId: number, input: LabelUpsertInput }) => api.labelsUpdate(labelId, input),
    onSuccess: invalidateLabelWrites
  }));
  const deleteLabel = proxyRefs(useMutation({
    mutationFn: async (labelId: number) => api.labelsDelete(labelId),
    onSuccess: invalidateLabelWrites
  }));

  return { createLabel, updateLabel, deleteLabel };
};
