import type { ButtonProps } from "@nuxt/ui";
import type { AppUpdateErrorCodeDto, AppUpdatePhaseDto, AppUpdateStateDto, AppUpdateStatusDto } from "~/generated/bindings";
import { relaunch } from "@tauri-apps/plugin-process";
import { inc } from "semver";

export type AppUpdateRowControl
  = | { kind: "none" }
    | { kind: "progress", value: number | null }
    | {
      kind: "button"
      props: Pick<ButtonProps, "label" | "icon" | "color" | "variant" | "loading" | "disabled" | "onClick">
    };

export interface AppUpdateRow {
  title: string
  titleProps: AppUpdateRowTitleProps
  description: string
  error: string | null
  control: AppUpdateRowControl
}

interface AppUpdateRowTitleProps {
  class?: string
  title?: string
  onClick?: (event: MouseEvent) => void
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
  const {
    updateRowMock,
    updateRowMockingActive,
    updateRowTitleProps
  } = useDevAppUpdateRowMock(() => appUpdateState.value?.current_version ?? null);

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
    const mockState = updateRowMock.value;
    const state = mockState ?? appUpdateState.value;
    const title = state == null ? "Version" : `Version ${state.current_version}`;
    const titleProps = updateRowTitleProps.value;
    const stateQueryFailed = mockState == null && stateQuery.isError;
    const commandErrorValue = mockState == null ? commandError.value : null;
    const checkPending = mockState == null && unref(checkForUpdates.isPending);
    const installPending = mockState == null && unref(installPendingUpdateAndRestart.isPending);
    const restartPendingValue = mockState == null && restartPending.value;

    if (state == null) {
      return {
        title,
        titleProps,
        description: stateQueryFailed ? "Update status is unavailable." : "Checking for updates...",
        error: commandErrorValue ?? getStateErrorReason(null, stateQueryFailed),
        control: { kind: "none" }
      };
    }

    const status = state.status;
    const isChecking = status.kind === "checking" || (checkPending && (status.kind === "idle" || status.kind === "up_to_date" || status.kind === "error"));
    const isDownloading = status.kind === "downloading";
    const isInstalling = installPending || status.kind === "installing";
    const isRestarting = restartPendingValue;
    const error = isChecking || isDownloading || isInstalling || isRestarting
      ? null
      : commandErrorValue ?? getStateErrorReason(status, stateQueryFailed);

    if (!state.supports_updates) {
      return {
        title,
        titleProps,
        description: "Updates are disabled for this installation.",
        error: null,
        control: { kind: "none" }
      };
    }

    if (isChecking) {
      return {
        title,
        titleProps,
        description: "Checking for updates...",
        error,
        control: checkButton(true)
      };
    }

    if (isDownloading) {
      return {
        title,
        titleProps,
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
        titleProps,
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
          titleProps,
          description: "You have the latest version.",
          error,
          control: checkButton(false)
        };
      case "downloaded":
        return {
          title,
          titleProps,
          description: `Version ${status.update.version} is ready to install.`,
          error,
          control: updateAndRestartButton(false)
        };
      case "installed":
        return {
          title,
          titleProps,
          description: `Version ${status.update.version} has been installed. Restart to finish updating.`,
          error,
          control: restartButton(restartPendingValue)
        };
      case "error":
        return {
          title,
          titleProps,
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
      props: {
        label: "Check for updates",
        icon: "i-lucide-refresh-cw",
        color: "neutral",
        variant: "subtle",
        ...getButtonOperationProps(loading, runCheckForUpdates)
      }
    };
  }

  function updateAndRestartButton(loading: boolean): AppUpdateRowControl {
    return {
      kind: "button",
      props: {
        label: "Update and restart Worth",
        icon: "i-lucide-download",
        color: "primary",
        ...getButtonOperationProps(loading, runUpdateAndRestart)
      }
    };
  }

  function restartButton(loading: boolean): AppUpdateRowControl {
    return {
      kind: "button",
      props: {
        label: "Restart Worth",
        icon: "i-lucide-rotate-cw",
        color: "primary",
        ...getButtonOperationProps(loading, runRestartWorth)
      }
    };
  }

  function getButtonOperationProps(loading: boolean, onClick: () => void | Promise<void>) {
    return {
      loading,
      disabled: loading,
      onClick: updateRowMockingActive.value ? () => {} : onClick
    };
  }

  return { updateRow };
}

function useDevAppUpdateRowMock(currentVersion: () => string | null) {
  if (!import.meta.dev) {
    return {
      updateRowMock: computed<AppUpdateStateDto | null>(() => null),
      updateRowMockingActive: computed(() => false),
      updateRowTitleProps: computed<AppUpdateRowTitleProps>(() => ({}))
    };
  }

  const updateRowMockingActive = useState("appUpdateRowMockingActive", () => false);
  const updateRowMockIndex = useState("appUpdateRowMockIndex", () => 0);
  const updateRowMocks = computed(() => getAppUpdateRowMocks(currentVersion() ?? "0.0.0"));
  const updateRowMock = computed(() => updateRowMockingActive.value
    ? updateRowMocks.value[updateRowMockIndex.value] ?? null
    : null);
  const updateRowTitleProps = computed<AppUpdateRowTitleProps>(() => ({
    class: "cursor-pointer select-none",
    title: "Click to preview the next update row state",
    onClick: (event) => {
      event.preventDefault();
      cycleUpdateRowMock();
    }
  }));

  function cycleUpdateRowMock() {
    if (!updateRowMockingActive.value) {
      updateRowMockingActive.value = true;
      updateRowMockIndex.value = 0;
      return;
    }

    if (updateRowMockIndex.value >= updateRowMocks.value.length - 1) {
      updateRowMockingActive.value = false;
      updateRowMockIndex.value = 0;
      return;
    }

    updateRowMockIndex.value += 1;
  }

  return { updateRowMock, updateRowMockingActive, updateRowTitleProps };
}

function getAppUpdateRowMocks(currentVersion: string): AppUpdateStateDto[] {
  const mockUpdate = {
    version: inc(currentVersion, "patch") ?? "9.9.9",
    current_version: currentVersion,
    target: "",
    body: null,
    date: null
  };
  const updatedAt = new Date().toISOString();

  return [
    getMockUpdateState(currentVersion, updatedAt, { kind: "up_to_date", check_mode: "user" }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "checking", check_mode: "user" }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "downloading", check_mode: "user", update: mockUpdate, downloaded_bytes: 60, total_bytes: 100 }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "installing", check_mode: "user", update: mockUpdate }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "downloaded", check_mode: "user", update: mockUpdate }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "installed", check_mode: "user", update: mockUpdate }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "error", check_mode: "user", phase: "checking", code: "network", message: "Network error", update: null }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "error", check_mode: "user", phase: "downloading", code: "signature", message: "Signature error", update: mockUpdate }),
    getMockUpdateState(currentVersion, updatedAt, { kind: "error", check_mode: "user", phase: "installing", code: "install", message: "Install error", update: mockUpdate })
  ];
}

function getMockUpdateState(currentVersion: string, updatedAt: string, status: AppUpdateStatusDto): AppUpdateStateDto {
  return {
    current_version: currentVersion,
    status,
    checked_at: updatedAt,
    updated_at: updatedAt,
    revision: 1,
    supports_updates: true
  };
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
