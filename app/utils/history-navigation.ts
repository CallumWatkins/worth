import type { Ref } from "vue";
import type { Router } from "vue-router";
import type { NavigationLayerOptions } from "../composables/useNavigationLayers";
import { computed, ref, shallowRef, toValue } from "vue";

interface HistoryNavigationLayers {
  getTopLayer: () => NavigationLayerOptions | null
  closeTopLayerForBack: () => Promise<"idle" | "handled" | "blocked">
}

/** One coordinator for title-bar buttons, keyboard shortcuts and mouse history inputs. */
export function createHistoryNavigation(
  router: Router,
  layers: HistoryNavigationLayers,
  confirmationOpen: Ref<boolean>,
  cancelConfirmation: () => boolean
) {
  const history = router.options.history;
  const state = shallowRef(history.state);
  const navigating = ref(false);
  const closingLayer = ref(false);
  const topLayer = computed(() => layers.getTopLayer());

  // History availability stays visually stable while a traversal is in flight.
  // The separate input guards still prevent repeated or conflicting requests.
  const backAvailable = computed(() => {
    if (confirmationOpen.value) return true;
    if (topLayer.value) return !toValue(topLayer.value.pending);
    return typeof state.value.back === "string";
  });
  const forwardAvailable = computed(() => !confirmationOpen.value && !topLayer.value
    && typeof state.value.forward === "string");
  const canGoBack = computed(() => confirmationOpen.value
    || (!navigating.value && !closingLayer.value && backAvailable.value));
  const canGoForward = computed(() => !navigating.value && !closingLayer.value && forwardAvailable.value);

  // A rejected pop is rolled back asynchronously by Vue Router. Keep the last
  // committed state until its compensating popstate arrives (it skips router hooks).
  const sync = () => {
    if (history.location !== router.currentRoute.value.fullPath) return;
    state.value = { ...history.state };
    navigating.value = false;
  };

  const back = async () => {
    if (cancelConfirmation()) return;
    if (!canGoBack.value) return;
    if (topLayer.value) {
      closingLayer.value = true;
      try {
        await layers.closeTopLayerForBack();
      } finally {
        closingLayer.value = false;
      }
      return;
    }
    navigating.value = true;
    router.back();
  };

  const forward = () => {
    if (!canGoForward.value) return;
    navigating.value = true;
    router.forward();
  };

  const removeBeforeGuard = router.beforeEach(() => {
    navigating.value = true;
  });
  const removeAfterGuard = router.afterEach(sync);
  const removeErrorHandler = router.onError(sync);

  return {
    backAvailable,
    forwardAvailable,
    canGoBack,
    canGoForward,
    back,
    forward,
    // Run after the router's own popstate listener, including history rollbacks.
    onPopState: () => {
      navigating.value = true;
      sync();
    },
    dispose: () => {
      removeBeforeGuard();
      removeAfterGuard();
      removeErrorHandler();
    }
  };
}
