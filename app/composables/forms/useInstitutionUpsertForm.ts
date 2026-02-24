export function useInstitutionUpsertForm() {
  const defaults: Partial<InstitutionFormInputValues> = {
  };

  const state = reactive<Partial<InstitutionFormInputValues>>({
    ...defaults
  });

  function reset() {
    Object.assign(state, defaults);
  }

  function hydrateFromInstitution(name: string) {
    state.name = name;
  }

  return {
    state,
    reset,
    hydrateFromInstitution
  };
}
