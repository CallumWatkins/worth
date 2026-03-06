// This middleware is used to detect whether the user navigated to an account page from the
// institution that it belongs to. This information is then used to dynamically adjust the
// breadcrumb items to better match the navigation context.

export interface AccountBreadcrumbContext {
  institutionId: number
  accountId: number
}

export default defineNuxtRouteMiddleware((to, from) => {
  const context = useState<AccountBreadcrumbContext | null>("accountBreadcrumbContext", () => null);

  const toAccountMatch = to.path.match(/^\/accounts\/(\d+)/);
  if (!toAccountMatch) {
    return;
  }

  const toAccountId = Number(toAccountMatch[1]);

  const fromInstitutionMatch = from.path.match(/^\/institutions\/(\d+)/);
  if (fromInstitutionMatch) {
    const fromInstitutionId = Number(fromInstitutionMatch[1]);
    // Navigating from an institution page to an account page
    context.value = {
      institutionId: fromInstitutionId,
      accountId: toAccountId
    };
    return;
  }

  const fromAccountMatch = from.path.match(/^\/accounts\/(\d+)/);
  if (fromAccountMatch) {
    const fromAccountId = Number(fromAccountMatch[1]);
    if (fromAccountId === toAccountId) {
      // Preserve institution context since we are navigating within the same account
      return;
    }
  }

  context.value = null;
});
