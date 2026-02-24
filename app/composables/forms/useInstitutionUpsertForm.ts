export function useInstitutionUpsertForm() {
  const state = reactive<Partial<InstitutionFormInputValues>>({
    name: ""
  });

  function reset() {
    state.name = "";
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
