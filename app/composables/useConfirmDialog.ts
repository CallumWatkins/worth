import { LazyConfirmDialog } from "#components";

let activeConfirmDialogCancel: (() => void) | null = null;

export interface ConfirmDialogOptions {
  title: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
}

export const useConfirmDialog = () => {
  const overlay = useOverlay();

  return async (options: ConfirmDialogOptions): Promise<boolean> => {
    const modal = overlay.create(LazyConfirmDialog, {
      destroyOnClose: true,
      props: options
    });

    const opened = modal.open();
    const cancel = () => modal.close(false);
    activeConfirmDialogCancel = cancel;

    try {
      // eslint-disable-next-line ts/no-unsafe-return
      return await opened.result;
    } finally {
      if (activeConfirmDialogCancel === cancel) {
        activeConfirmDialogCancel = null;
      }
    }
  };
};

export const cancelActiveConfirmDialog = () => {
  if (!activeConfirmDialogCancel) return false;

  activeConfirmDialogCancel();
  activeConfirmDialogCancel = null;
  return true;
};
