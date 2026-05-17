import type { AppUpdateStateDto, AppUpdateStatusDto } from "~/generated/bindings";
import { useQueryClient } from "@tanstack/vue-query";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";

const STARTUP_UPDATE_TOAST_ID = "startup-update";

export default defineNuxtPlugin({
  name: "app-updates-listener",
  dependsOn: ["vue-query"],
  async setup() {
    const api = useApi();
    const appUpdateState = useAppUpdateState();
    const queryClient = useQueryClient();
    const toast = useToast();
    let startupUpdateToastShown = false;
    let activeStartupUpdateToastKey: string | null = null;
    let toastActionRunning = false;

    function setState(state: AppUpdateStateDto) {
      if (appUpdateState.value != null && state.revision <= appUpdateState.value.revision) return;

      appUpdateState.value = state;
      queryClient.setQueryData(queryKeys.appUpdates.state(), state);
      handleStartupUpdateToast(state);
    }

    function handleStartupUpdateToast(state: AppUpdateStateDto) {
      const update = getStartupActionableUpdate(state.status);
      const key = update == null ? null : `${update.kind}:${update.version}`;

      if (activeStartupUpdateToastKey != null && key !== activeStartupUpdateToastKey && !toastActionRunning) {
        toast.remove(STARTUP_UPDATE_TOAST_ID);
        activeStartupUpdateToastKey = null;
      }

      if (startupUpdateToastShown || update == null) return;

      startupUpdateToastShown = true;
      activeStartupUpdateToastKey = key;
      toast.add({
        id: STARTUP_UPDATE_TOAST_ID,
        title: update.kind === "downloaded" ? "Update available" : "Update installed",
        description: update.kind === "downloaded"
          ? `A new version of Worth (${update.version}) is available to install.`
          : `A new version of Worth (${update.version}) has been installed.`,
        icon: "i-lucide-download",
        color: "neutral",
        type: "background",
        duration: 0,
        actions: [{
          label: update.kind === "downloaded" ? "Update and restart Worth" : "Restart Worth",
          color: "neutral",
          variant: "outline",
          onClick: async () => {
            toastActionRunning = true;

            try {
              if (update.kind === "downloaded") {
                try {
                  const state = await api.appUpdatesInstallPendingAndRestart();
                  setState(state);
                  if (state.status.kind === "error" && state.status.phase === "installing") {
                    showActionErrorToast("install");
                  }
                } catch {
                  showActionErrorToast("install");
                }
              } else {
                try {
                  await relaunch();
                } catch {
                  showActionErrorToast("restart");
                }
              }
            } finally {
              toastActionRunning = false;
            }
          }
        }],
        "onUpdate:open": (open: boolean) => {
          if (!open && activeStartupUpdateToastKey === key) {
            activeStartupUpdateToastKey = null;
          }
        }
      });
    }

    function showActionErrorToast(action: "install" | "restart") {
      toast.add({
        title: action === "install" ? "Update failed" : "Restart failed",
        description: action === "install" ? "Worth could not install the update." : "Worth could not restart.",
        icon: "i-lucide-circle-alert",
        color: "error"
      });
    }

    // Subscribe to state changes
    await listen<AppUpdateStateDto>(APP_UPDATE_STATE_EVENT, (event) => {
      setState(event.payload);
    });

    try {
      // Fetch initial state
      setState(await api.appUpdatesStateGet());
    } catch {
    }
  }
});

function getStartupActionableUpdate(status: AppUpdateStatusDto): { kind: "downloaded" | "installed", version: string } | null {
  if (status.kind !== "downloaded" && status.kind !== "installed") return null;
  if (status.check_mode !== "startup") return null;
  return {
    kind: status.kind,
    version: status.update.version
  };
}
