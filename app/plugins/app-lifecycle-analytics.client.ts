import { getVersion } from "@tauri-apps/api/app";
import { compare, valid } from "semver";

const LAST_LAUNCHED_APP_VERSION_KEY = "worth:last-launched-app-version";

export default defineNuxtPlugin({
  name: "app-lifecycle-analytics",
  dependsOn: ["settings"],
  async setup() {
    const { captureAnalyticsEvent } = useAnalytics();
    const currentAppVersion = await getVersion();
    const previousAppVersion = localStorage.getItem(LAST_LAUNCHED_APP_VERSION_KEY);

    if (previousAppVersion == null) {
      captureAnalyticsEvent("app:app_install");
    } else {
      const normalizedCurrentAppVersion = valid(currentAppVersion);
      const normalizedPreviousAppVersion = valid(previousAppVersion);

      const versionComparison = normalizedCurrentAppVersion != null && normalizedPreviousAppVersion != null
        ? compare(normalizedCurrentAppVersion, normalizedPreviousAppVersion)
        : 0;

      if (versionComparison > 0) {
        captureAnalyticsEvent("app:app_upgrade", { previous_app_version: previousAppVersion });
      } else if (versionComparison < 0) {
        captureAnalyticsEvent("app:app_downgrade", { previous_app_version: previousAppVersion });
      }
    }

    captureAnalyticsEvent("app:app_open");
    localStorage.setItem(LAST_LAUNCHED_APP_VERSION_KEY, currentAppVersion);
  }
});
