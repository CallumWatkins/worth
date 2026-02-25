import { LazyConfirmDialog } from "#components";

export interface ConfirmDialogOptions {
  title: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
}

export const useConfirmDialog = () => {
  const overlay = useOverlay();

  return (options: ConfirmDialogOptions): Promise<boolean> => {
    const modal = overlay.create(LazyConfirmDialog, {
      destroyOnClose: true,
      props: options
    });

    const opened = modal.open();
    if (opened && typeof opened === "object" && "result" in opened) {
      return (opened as { result: Promise<boolean> }).result;
    }

    return opened as Promise<boolean>;
  };
};
