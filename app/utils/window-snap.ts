export interface WindowSnapBounds {
  x: number
  y: number
  width: number
  height: number
}

/**
 * Track the rendered button, including position-only layout changes and WebView zoom.
 * IPC is sent only when physical bounds change. Returning null removes the target.
 */
export function trackWindowSnapButton(button: HTMLButtonElement, update: (bounds: WindowSnapBounds | null) => void) {
  let previous: string | undefined;
  let frame: number;
  let stopped = false;

  const measure = () => {
    if (stopped) return;
    let bounds: WindowSnapBounds | null = null;
    if (button.isConnected && !button.disabled && button.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true })) {
      const rect = button.getBoundingClientRect();
      const scale = window.devicePixelRatio;
      const x = Math.round(rect.left * scale);
      const y = Math.round(rect.top * scale);
      const width = Math.round(rect.right * scale) - x;
      const height = Math.round(rect.bottom * scale) - y;
      if (width > 0 && height > 0) bounds = { x, y, width, height };
    }
    const key = JSON.stringify(bounds);
    if (key !== previous) {
      previous = key;
      update(bounds);
    }
    frame = requestAnimationFrame(measure);
  };
  measure();

  return () => {
    stopped = true;
    cancelAnimationFrame(frame);
    update(null);
  };
}
