import type { AppUpdateStateDto } from "~/generated/bindings";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";

export const APP_UPDATE_STATE_EVENT = "worth://updates/state";

export function useAppUpdateState() {
  return useState<AppUpdateStateDto | null>("appUpdateState", () => null);
}

export function useAppUpdatesManager() {
  const api = useApi();
  const queryClient = useQueryClient();
  const appUpdateState = useAppUpdateState();

  const stateQuery = proxyRefs(useQuery({
    queryKey: queryKeys.appUpdates.state(),
    queryFn: api.appUpdatesStateGet,
    staleTime: Infinity
  }));

  function setState(state: AppUpdateStateDto) {
    appUpdateState.value = state;
    queryClient.setQueryData(queryKeys.appUpdates.state(), state);
  }

  watch(() => stateQuery.data, (data) => {
    if (data != null) setState(data);
  }, { immediate: true });

  const checkForUpdates = proxyRefs(useMutation({
    mutationFn: api.appUpdatesCheck,
    onSuccess: setState
  }));

  const installPendingUpdateAndRestart = proxyRefs(useMutation({
    mutationFn: api.appUpdatesInstallPendingAndRestart,
    onSuccess: setState
  }));

  return {
    appUpdateState,
    stateQuery,
    checkForUpdates,
    installPendingUpdateAndRestart
  };
}
