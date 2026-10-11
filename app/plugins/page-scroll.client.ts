export default defineNuxtPlugin((nuxtApp) => {
  // Nuxt installs its final scroll behavior during app creation.
  nuxtApp.hook("app:mounted", () => {
    const router = useRouter();
    const history = router.options.history;
    const scrollBehavior = router.options.scrollBehavior;
    const positions = new Map<number, { left: number, top: number }>();
    let currentPosition = Number(history.state.position);

    // Vue Router records window scroll offsets; the shell owns page scrolling.
    // Keep positions per history entry for Back and Forward.
    const removeBeforeGuard = router.beforeEach(() => {
      const viewport = document.getElementById("app-scroll");
      if (!viewport) return;
      positions.set(currentPosition, {
        left: viewport.scrollLeft,
        top: viewport.scrollTop
      });
    });
    const removeAfterGuard = router.afterEach((_to, _from, failure) => {
      if (!failure) currentPosition = Number(history.state.position);
    });

    router.options.scrollBehavior = async (to, from, savedPosition): Promise<false> => {
      const position = savedPosition
        ? positions.get(Number(history.state.position)) ?? savedPosition
        : null;
      // Retain Nuxt's page-transition timing, hash links and scrollToTop policy.
      const target = await scrollBehavior?.(to, from, position);
      if (target == null || target === false || router.currentRoute.value.fullPath !== to.fullPath) return false;
      // Resolve after navigation because fatal error recovery replaces the shell.
      const viewport = document.getElementById("app-scroll");
      if (!viewport) return false;

      if ("el" in target) {
        const element = typeof target.el === "string"
          ? target.el.startsWith("#")
            ? document.getElementById(target.el.slice(1))
            : document.querySelector(target.el)
          : target.el;
        if (element) {
          const bounds = element.getBoundingClientRect();
          const viewportBounds = viewport.getBoundingClientRect();
          viewport.scrollTo({
            left: viewport.scrollLeft + bounds.left - viewportBounds.left - (target.left ?? 0),
            top: viewport.scrollTop + bounds.top - viewportBounds.top - (target.top ?? 0),
            behavior: target.behavior
          });
        }
      } else {
        viewport.scrollTo(target);
      }
      return false;
    };

    nuxtApp.vueApp.onUnmount(() => {
      removeBeforeGuard();
      removeAfterGuard();
      router.options.scrollBehavior = scrollBehavior;
    });
  });
});
