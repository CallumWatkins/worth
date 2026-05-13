import { getVersion } from "@tauri-apps/api/app";

export default defineNuxtPlugin({
  name: "posthog-super-properties",
  dependsOn: ["posthog-client"],
  async setup() {
    usePostHog()?.register({
      surface: "desktop_app",
      surface_version: await getVersion()
    });
  }
});
