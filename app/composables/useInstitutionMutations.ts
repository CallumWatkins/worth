import type { InstitutionUpsertInput } from "~/generated/bindings";

import { useMutation, useQueryClient } from "@tanstack/vue-query";

export const useInstitutionMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();

  const invalidateInstitutionWrites = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() })
    ]);
  };

  const createInstitution = proxyRefs(useMutation({
    mutationFn: async (input: InstitutionUpsertInput) => api.institutionsCreate(input),
    onSuccess: invalidateInstitutionWrites
  }));

  const updateInstitution = proxyRefs(useMutation({
    mutationFn: async ({ institutionId, input }: { institutionId: number, input: InstitutionUpsertInput }) =>
      api.institutionsUpdate(institutionId, input),
    onSuccess: invalidateInstitutionWrites
  }));

  const deleteInstitution = proxyRefs(useMutation({
    mutationFn: async (institutionId: number) => api.institutionsDelete(institutionId),
    onSuccess: invalidateInstitutionWrites
  }));

  return {
    createInstitution,
    updateInstitution,
    deleteInstitution
  };
};
