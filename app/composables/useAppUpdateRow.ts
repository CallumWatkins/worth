import type { ButtonProps } from "@nuxt/ui";
import type { AppUpdateErrorCodeDto, AppUpdatePhaseDto, AppUpdateStatusDto } from "~/generated/bindings";
import { relaunch } from "@tauri-apps/plugin-process";

export type AppUpdateRowControl
  = | { kind: "none" }
    | { kind: "progress", value: number | null }
    | {
      kind: "button"
      action: "check" | "update_and_restart" | "restart"
      props: Pick<ButtonProps, "label" | "icon" | "color" | "variant" | "loading" | "disabled" | "onClick">
    };

export interface AppUpdateRow {
  title: string
  description: string
  error: string | null
  control: AppUpdateRowControl
}

export function useAppUpdateRow() {
  const {
    appUpdateState,
    stateQuery,
    checkForUpdates,
    installPendingUpdateAndRestart
  } = useAppUpdatesManager();

  const commandError = ref<string | null>(null);
  const restartPending = ref(false);

  async function runCheckForUpdates() {
    commandError.value = null;

    try {
      await checkForUpdates.mutateAsync();
    } catch (error) {
      commandError.value = getErrorMessage(error, "Failed to check for updates");
    }
  }

  async function runUpdateAndRestart() {
    commandError.value = null;

    try {
      await installPendingUpdateAndRestart.mutateAsync();
    } catch (error) {
      commandError.value = getErrorMessage(error, "Failed to install update");
    }
  }

  async function runRestartWorth() {
    commandError.value = null;
    restartPending.value = true;

    try {
      await relaunch();
    } catch (error) {
      commandError.value = getErrorMessage(error, "Failed to restart Worth");
    } finally {
      restartPending.value = false;
    }
  }

  const updateRow = computed<AppUpdateRow>(() => {
    const state = appUpdateState.value;
    const title = state == null ? "Version" : `Version ${state.current_version}`;
    const checkPending = unref(checkForUpdates.isPending);
    const installPending = unref(installPendingUpdateAndRestart.isPending);

    if (state == null) {
      return {
        title,
        description: stateQuery.isError ? "Update status is unavailable." : "Checking for updates...",
        error: commandError.value ?? getStateErrorReason(null, stateQuery.isError),
        control: { kind: "none" }
      };
    }

    const status = state.status;
    const isChecking = status.kind === "checking" || (checkPending && (status.kind === "idle" || status.kind === "up_to_date" || status.kind === "error"));
    const isDownloading = status.kind === "downloading";
    const isInstalling = installPending || status.kind === "installing";
    const isRestarting = restartPending.value;
    const error = isChecking || isDownloading || isInstalling || isRestarting
      ? null
      : commandError.value ?? getStateErrorReason(status, stateQuery.isError);

    if (!state.supports_updates) {
      return {
        title,
        description: "Updates are disabled for this installation.",
        error: null,
        control: { kind: "none" }
      };
    }

    if (isChecking) {
      return {
        title,
        description: "Checking for updates...",
        error,
        control: checkButton(true)
      };
    }

    if (isDownloading) {
      return {
        title,
        description: "Downloading update...",
        error,
        control: {
          kind: "progress",
          value: status.total_bytes == null || status.total_bytes <= 0
            ? null
            : Math.min(100, Math.round((status.downloaded_bytes / status.total_bytes) * 100))
        }
      };
    }

    if (isInstalling) {
      return {
        title,
        description: "Installing update...",
        error,
        control: { kind: "progress", value: null }
      };
    }

    switch (status.kind) {
      case "idle":
      case "up_to_date":
        return {
          title,
          description: "You have the latest version.",
          error,
          control: checkButton(false)
        };
      case "downloaded":
        return {
          title,
          description: `Version ${status.update.version} is ready to install.`,
          error,
          control: updateAndRestartButton(false)
        };
      case "installed":
        return {
          title,
          description: `Version ${status.update.version} has been installed. Restart to finish updating.`,
          error,
          control: restartButton(restartPending.value)
        };
      case "error":
        return {
          title,
          description: getErrorStatusDescription(status.phase, status.update?.version ?? null),
          error,
          control: status.phase === "installing" && status.update != null
            ? updateAndRestartButton(false)
            : checkButton(false)
        };
    }
  });

  function checkButton(loading: boolean): AppUpdateRowControl {
    return {
      kind: "button",
      action: "check",
      props: {
        label: "Check for updates",
        icon: "i-lucide-refresh-cw",
        variant: "subtle",
        loading,
        disabled: loading,
        onClick: runCheckForUpdates
      }
    };
  }

  function updateAndRestartButton(loading: boolean): AppUpdateRowControl {
    return {
      kind: "button",
      action: "update_and_restart",
      props: {
        label: "Update and restart Worth",
        icon: "i-lucide-download",
        color: "primary",
        loading,
        disabled: loading,
        onClick: runUpdateAndRestart
      }
    };
  }

  function restartButton(loading: boolean): AppUpdateRowControl {
    return {
      kind: "button",
      action: "restart",
      props: {
        label: "Restart Worth",
        icon: "i-lucide-rotate-cw",
        color: "primary",
        loading,
        disabled: loading,
        onClick: runRestartWorth
      }
    };
  }

  return { updateRow };
}

function getStateErrorReason(
  status: AppUpdateStatusDto | null,
  stateQueryFailed: boolean
) {
  if (status?.kind === "error") return getUpdateErrorReason(status.code);
  if (stateQueryFailed) return "Worth could not load update status.";
  return null;
}

function getErrorStatusDescription(phase: AppUpdatePhaseDto, version: string | null) {
  if (phase === "installing" && version != null) return `Version ${version} is ready to install.`;
  if (phase === "downloading" && version != null) return `Version ${version} is available.`;
  return "Update status is unavailable.";
}

function getUpdateErrorReason(code: AppUpdateErrorCodeDto) {
  switch (code) {
    case "configuration":
      return "Updates are not configured correctly for this installation.";
    case "manifest":
      return "Worth could not read the update information.";
    case "network":
      return "Worth could not connect to the update service.";
    case "signature":
      return "Worth could not verify the downloaded update.";
    case "install":
      return "Worth could not install the update.";
    case "unsupported":
      return "Updates are not available on this device.";
    case "no_pending_update":
      return "There is no update ready to install.";
    case "unknown":
      return "Something went wrong while updating Worth.";
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}
