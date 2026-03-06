import { LazyConfirmDialog } from "#components";

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
    // eslint-disable-next-line ts/no-unsafe-return
    return opened.result;
  };
};
