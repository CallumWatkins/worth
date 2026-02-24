import type { InstitutionUpsertInput } from "~/bindings";

import { useMutation, useQueryClient } from "@tanstack/vue-query";

export const useInstitutionMutations = () => {
  const api = useApi();
  const queryClient = useQueryClient();

  const invalidateInstitutionWrites = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.accounts.prefixes.root() }),
      queryClient.invalidateQueries({ queryKey: queryKeys.search.prefixes.root() })
    ]);
  };

  const createInstitution = useMutation({
    mutationFn: (input: InstitutionUpsertInput) => api.institutionsCreate(input),
    onSuccess: invalidateInstitutionWrites
  });

  const updateInstitution = useMutation({
    mutationFn: ({ institutionId, input }: { institutionId: number, input: InstitutionUpsertInput }) =>
      api.institutionsUpdate(institutionId, input),
    onSuccess: invalidateInstitutionWrites
  });

  return {
    createInstitution,
    updateInstitution
  };
};
