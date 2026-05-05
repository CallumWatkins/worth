import type { AppSettingsDto, AppSettingsUpdateInput } from "~/generated/bindings";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";

export const themePreferenceItems = [
  { label: "System", value: "system", icon: "i-lucide-monitor" },
  { label: "Light", value: "light", icon: "i-lucide-sun" },
  { label: "Dark", value: "dark", icon: "i-lucide-moon" }
] as const;

export function useNullableSettings() {
  return useState<AppSettingsDto | null>("appSettings", () => null);
}

export function useSettings() {
  const state = useNullableSettings();

  return computed<AppSettingsDto>({
    get: () => {
      if (state.value == null) {
        throw new Error("App settings have not loaded");
      }

      return state.value;
    },
    set: (value) => {
      state.value = value;
    }
  });
}

export function useSettingsManager() {
  const api = useApi();
  const queryClient = useQueryClient();
  const settings = useSettings();

  const settingsQuery = proxyRefs(useQuery({
    queryKey: queryKeys.settings.get(),
    queryFn: api.settingsGet,
    staleTime: Infinity
  }));

  watch(() => settingsQuery.data, (data) => {
    if (data != null) settings.value = data;
  }, { immediate: true });

  const updateSettings = proxyRefs(useMutation({
    mutationFn: async (input: AppSettingsUpdateInput) => api.settingsUpdate(input),
    onSuccess: async (updated) => {
      const previousCurrency = settings.value.default_display_currency_code;
      settings.value = updated;
      queryClient.setQueryData(queryKeys.settings.get(), updated);

      if (previousCurrency !== updated.default_display_currency_code) {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.dashboard.prefixes.root() }),
          queryClient.invalidateQueries({ queryKey: queryKeys.institutions.prefixes.root() })
        ]);
      }
    }
  }));

  async function updateSetting(patch: Partial<AppSettingsUpdateInput>) {
    return updateSettings.mutateAsync(patch);
  }

  return {
    settings,
    settingsQuery,
    updateSettings,
    updateSetting
  };
}
