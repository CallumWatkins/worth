import type { UnlistenFn } from "@tauri-apps/api/event";
import type { WindowSnapBounds } from "~/utils/window-snap";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { trackWindowSnapButton } from "~/utils/window-snap";

// Serialize across component remounts so an old shell cannot detach a new target.
let updates = Promise.resolve();
function updateBounds(bounds: WindowSnapBounds | null) {
  updates = updates
    .then(async () => invoke<void>("plugin:window-snap|set_bounds", { bounds }))
    .catch((error) => {
      console.error("Could not update Windows Snap target", error);
    });
}

export function useWindowSnap(buttonId: string) {
  const state = ref({ hovered: false, pressed: false });
  let disposed = false;
  let stop: (() => void) | undefined;
  let unlisten: UnlistenFn | undefined;

  onMounted(async () => {
    if (!isTauri() || !navigator.userAgent.includes("Windows")) return;
    const listener = await getCurrentWindow().listen<{ hovered: boolean, pressed: boolean }>("worth://window-snap", ({ payload }) => {
      state.value = payload;
    });
    if (disposed) {
      listener();
      return;
    }
    unlisten = listener;
    const button = document.getElementById(buttonId);
    if (button instanceof HTMLButtonElement) stop = trackWindowSnapButton(button, updateBounds);
  });

  onBeforeUnmount(() => {
    disposed = true;
    stop?.();
    unlisten?.();
  });

  return state;
}
