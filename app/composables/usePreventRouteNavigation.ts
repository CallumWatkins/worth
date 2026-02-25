interface UsePreventRouteNavigationOptions {
  isSubmitting: Ref<boolean>
  isDirty: Ref<boolean>
  title?: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
}

export function usePreventRouteNavigation(options: UsePreventRouteNavigationOptions) {
  const confirm = useConfirmDialog();
  let pendingConfirmation: Promise<boolean> | null = null;

  const requestDiscardConfirmation = () => {
    if (!pendingConfirmation) {
      pendingConfirmation = confirm({
        title: options.title ?? "Discard unsaved changes?",
        description: options.description ?? "Your changes will be lost if you leave this page.",
        confirmLabel: options.confirmLabel,
        cancelLabel: options.cancelLabel
      }).finally(() => {
        pendingConfirmation = null;
      });
    }

    return pendingConfirmation;
  };

  onBeforeRouteLeave(async () => {
    if (options.isSubmitting.value) return false;
    if (!options.isDirty.value) return true;

    return await requestDiscardConfirmation();
  });

  onBeforeRouteUpdate(async () => {
    if (options.isSubmitting.value) return false;
    if (!options.isDirty.value) return true;

    return await requestDiscardConfirmation();
  });
}
