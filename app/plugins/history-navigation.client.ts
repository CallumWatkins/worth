export default defineNuxtPlugin({
  name: "history-navigation",
  setup() {
    const router = useRouter();
    const {
      hasOpenLayers,
      closeTopLayerForBack,
      prepareLayersForRouteNavigation
    } = useNavigationLayers();
    const nuxtError = useError();
    const suppressedMouseButtons = new Set<number>();

    const isCatchAllRoute = (route: typeof router.currentRoute.value) => {
      return route.matched.some((record) => record.name === "all" || record.path.includes(":all"));
    };

    const suppressMouseButton = (button: number) => {
      suppressedMouseButtons.add(button);
      window.setTimeout(() => suppressedMouseButtons.delete(button), 500);
    };

    const preventHistoryEvent = (event: Event) => {
      event.preventDefault();
      event.stopImmediatePropagation();
    };

    const interceptBack = (event: Event) => {
      if (cancelActiveConfirmDialog()) {
        preventHistoryEvent(event);
        return true;
      }

      if (!hasOpenLayers()) return false;

      preventHistoryEvent(event);
      void closeTopLayerForBack();
      return true;
    };

    const interceptForward = (event: Event) => {
      if (!hasOpenLayers()) return false;

      preventHistoryEvent(event);
      return true;
    };

    const handleKeydown = (event: KeyboardEvent) => {
      if (!event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;

      if (event.key === "ArrowLeft") {
        interceptBack(event);
      } else if (event.key === "ArrowRight") {
        interceptForward(event);
      }
    };

    const handleMouseHistoryButton = (event: MouseEvent) => {
      if (event.button !== 3 && event.button !== 4) return;

      if (event.type !== "mousedown" && suppressedMouseButtons.has(event.button)) {
        preventHistoryEvent(event);
        return;
      }

      const intercepted = event.button === 3
        ? interceptBack(event)
        : interceptForward(event);

      if (intercepted) suppressMouseButton(event.button);
    };

    window.addEventListener("keydown", handleKeydown, { capture: true, passive: false });
    window.addEventListener("mousedown", handleMouseHistoryButton, { capture: true, passive: false });
    window.addEventListener("mouseup", handleMouseHistoryButton, { capture: true, passive: false });
    window.addEventListener("auxclick", handleMouseHistoryButton, { capture: true, passive: false });

    router.beforeEach(async () => {
      return prepareLayersForRouteNavigation();
    });

    router.afterEach((to) => {
      if (!nuxtError.value || isCatchAllRoute(to)) return;

      void clearError();
    });
  }
});
