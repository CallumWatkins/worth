import type { Form } from "@nuxt/ui";

export function useBackendValidationErrors<S>(form: Ref<Pick<Form<S>, "setErrors"> | null | undefined>) {
  return (error: unknown) => {
    const issues = extractValidationIssues(error);
    if (!issues.length) return false;

    form.value?.setErrors(issues.map((issue) => ({
      name: issue.field,
      message: issue.message
    })));
    return true;
  };
}
