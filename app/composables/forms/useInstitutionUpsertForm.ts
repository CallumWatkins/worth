import type { InstitutionDetailDto, InstitutionDto } from "~/generated/bindings";

export function useInstitutionUpsertForm() {
  const defaults: RequiredOrUndefined<InstitutionFormInputValues> = {
    name: undefined
  };

  const state = reactive<Partial<InstitutionFormInputValues>>({
    ...defaults
  });

  function reset() {
    Object.assign(state, defaults);
  }

  function hydrateFromInstitution(institution: InstitutionDto | InstitutionDetailDto) {
    state.name = institution.name;
  }

  return {
    state,
    reset,
    hydrateFromInstitution
  };
}
