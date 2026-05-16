import type { AppUpdateStateDto } from "~/generated/bindings";
import { useQueryClient } from "@tanstack/vue-query";
import { listen } from "@tauri-apps/api/event";

export default defineNuxtPlugin({
  name: "app-updates-listener",
  dependsOn: ["vue-query"],
  async setup() {
    const appUpdateState = useAppUpdateState();
    const queryClient = useQueryClient();

    await listen<AppUpdateStateDto>(APP_UPDATE_STATE_EVENT, (event) => {
      appUpdateState.value = event.payload;
      queryClient.setQueryData(queryKeys.appUpdates.state(), event.payload);
    });
  }
});
