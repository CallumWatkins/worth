export default defineNuxtPlugin({
  name: "posthog-super-properties",
  dependsOn: ["posthog-client"],
  setup() {
    usePostHog()?.register({ surface: "desktop_app" });
  }
});
