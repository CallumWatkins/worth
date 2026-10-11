export default defineNuxtPlugin({
  name: "history-navigation",
  setup(nuxtApp) {
    const router = useRouter();
    const layers = useNavigationLayers();
    const historyNavigation = createHistoryNavigation(router, layers, isConfirmDialogOpen, cancelActiveConfirmDialog);
    const nuxtError = useError();

    const isCatchAllRoute = (route: typeof router.currentRoute.value) => {
      return route.matched.some((record) => record.name === "all" || record.path.includes(":all"));
    };

    const preventHistoryEvent = (event: Event) => {
      event.preventDefault();
      event.stopImmediatePropagation();
    };

    const handleKeydown = (event: KeyboardEvent) => {
      if (!event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;

      if (event.key === "ArrowLeft") {
        preventHistoryEvent(event);
        void historyNavigation.back();
      } else if (event.key === "ArrowRight") {
        preventHistoryEvent(event);
        historyNavigation.forward();
      }
    };

    const handleMouseHistoryButton = (event: MouseEvent) => {
      if (event.button !== 3 && event.button !== 4) return;

      preventHistoryEvent(event);
      // WebViews can deliver all three events for one side-button press.
      if (event.type !== "mousedown") return;
      if (event.button === 3) void historyNavigation.back();
      else historyNavigation.forward();
    };

    window.addEventListener("keydown", handleKeydown, { capture: true, passive: false });
    window.addEventListener("mousedown", handleMouseHistoryButton, { capture: true, passive: false });
    window.addEventListener("mouseup", handleMouseHistoryButton, { capture: true, passive: false });
    window.addEventListener("auxclick", handleMouseHistoryButton, { capture: true, passive: false });
    window.addEventListener("popstate", historyNavigation.onPopState);

    const removeLayerGuard = router.beforeEach(async () => {
      return layers.prepareLayersForRouteNavigation();
    });

    const removeErrorGuard = router.afterEach((to, _from, failure) => {
      if (failure || !nuxtError.value || isCatchAllRoute(to)) return;

      void clearError();
    });

    nuxtApp.vueApp.onUnmount(() => {
      window.removeEventListener("keydown", handleKeydown, true);
      window.removeEventListener("mousedown", handleMouseHistoryButton, true);
      window.removeEventListener("mouseup", handleMouseHistoryButton, true);
      window.removeEventListener("auxclick", handleMouseHistoryButton, true);
      window.removeEventListener("popstate", historyNavigation.onPopState);
      removeLayerGuard();
      removeErrorGuard();
      historyNavigation.dispose();
    });

    return { provide: { historyNavigation } };
  }
});
