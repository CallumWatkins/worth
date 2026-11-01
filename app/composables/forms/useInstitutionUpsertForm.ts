import type { InstitutionDetailDto } from "~/generated/bindings";

export function useInstitutionUpsertForm() {
  const defaults = (): RequiredOrUndefined<InstitutionFormInputValues> => ({
    name: undefined,
    identifiers: []
  });

  const state = shallowReactive<Partial<InstitutionFormInputValues>>({
    ...defaults()
  });

  function reset() {
    Object.assign(state, defaults());
  }

  function hydrateFromInstitution(institution: InstitutionDetailDto) {
    state.name = institution.name;
    state.identifiers = institution.identifiers.map((identifier) => ({ ...identifier }));
  }

  function discardEmptyIdentifiers() {
    state.identifiers = state.identifiers?.filter(({ label, value }) => label.trim() !== "" || value.trim() !== "");
  }

  return {
    state,
    reset,
    discardEmptyIdentifiers,
    hydrateFromInstitution
  };
}
