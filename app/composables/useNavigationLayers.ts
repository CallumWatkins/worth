import type { MaybeRefOrGetter } from "vue";
import { nextTick, onScopeDispose, shallowReactive, toValue, watch } from "vue";

export interface NavigationLayerOptions {
  id?: string
  open: MaybeRefOrGetter<boolean>
  dirty?: MaybeRefOrGetter<boolean>
  pending?: MaybeRefOrGetter<boolean>
  close: () => void | Promise<void>
  discardTitle?: string
  discardDescription?: string
}

interface NavigationLayerRegistration extends NavigationLayerOptions {
  id: string
  openedOrder: number
}

export type NavigationLayerBackResult = "idle" | "handled" | "blocked";

const layers = shallowReactive<NavigationLayerRegistration[]>([]);
let nextLayerId = 0;
let nextOpenedOrder = 0;
let pendingDiscardConfirmation: Promise<boolean> | null = null;

const isLayerOpen = (layer: NavigationLayerRegistration) => toValue(layer.open);
const isLayerDirty = (layer: NavigationLayerRegistration) => toValue(layer.dirty) === true;
const isLayerPending = (layer: NavigationLayerRegistration) => toValue(layer.pending) === true;

const getOpenLayerRegistrations = () => {
  return layers
    .filter(isLayerOpen)
    .toSorted((left, right) => left.openedOrder - right.openedOrder);
};

const requestDiscardConfirmation = async (confirm: ReturnType<typeof useConfirmDialog>, layer: NavigationLayerRegistration) => {
  if (!pendingDiscardConfirmation) {
    pendingDiscardConfirmation = confirm({
      title: layer.discardTitle ?? "Discard changes and close?",
      description: layer.discardDescription ?? "Your changes will be lost if you close this dialog.",
      confirmLabel: "Discard changes",
      cancelLabel: "Keep editing"
    }).finally(() => {
      pendingDiscardConfirmation = null;
    });
  }

  return pendingDiscardConfirmation;
};

const closeLayer = async (layer: NavigationLayerRegistration) => {
  await layer.close();
  await nextTick();
};

export const useNavigationLayer = (options: NavigationLayerOptions) => {
  const layer: NavigationLayerRegistration = {
    ...options,
    id: options.id ?? `navigation-layer-${++nextLayerId}`,
    openedOrder: 0
  };

  const stopWatchingOpen = watch(
    () => toValue(layer.open),
    (open, wasOpen) => {
      if (open && !wasOpen) {
        layer.openedOrder = ++nextOpenedOrder;
      } else if (!open) {
        layer.openedOrder = 0;
      }
    },
    { immediate: true }
  );

  layers.push(layer);

  onScopeDispose(() => {
    stopWatchingOpen();
    const layerIndex = layers.indexOf(layer);
    if (layerIndex !== -1) layers.splice(layerIndex, 1);
  });

  return layer.id;
};

export const useNavigationLayers = () => {
  const confirm = useConfirmDialog();

  const getOpenLayers = () => [...getOpenLayerRegistrations()];
  const getTopLayer = () => getOpenLayerRegistrations().at(-1) ?? null;
  const hasOpenLayers = () => getTopLayer() != null;

  const closeTopLayerForBack = async (): Promise<NavigationLayerBackResult> => {
    const layer = getTopLayer();
    if (!layer) return "idle";
    if (isLayerPending(layer)) return "blocked";

    if (isLayerDirty(layer)) {
      const confirmed = await requestDiscardConfirmation(confirm, layer);
      if (!confirmed) return "handled";
    }

    await closeLayer(layer);
    return "handled";
  };

  const prepareLayersForRouteNavigation = async () => {
    for (const layer of getOpenLayerRegistrations().reverse()) {
      if (!isLayerOpen(layer)) continue;
      if (isLayerPending(layer)) return false;

      if (isLayerDirty(layer)) {
        const confirmed = await requestDiscardConfirmation(confirm, layer);
        if (!confirmed) return false;
      }

      await closeLayer(layer);
    }

    return true;
  };

  return {
    getOpenLayers,
    getTopLayer,
    hasOpenLayers,
    closeTopLayerForBack,
    prepareLayersForRouteNavigation
  };
};
