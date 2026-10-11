import type { PopoverContentProps } from "reka-ui";
import type { InjectionKey, ShallowRef } from "vue";
import { computed, inject } from "vue";

export const appViewportKey: InjectionKey<Readonly<ShallowRef<HTMLElement | null>>> = Symbol("app-viewport");

/** Preserve the shell's collision boundary when overriding a popup's content props. */
export function useAppPopupContent(options: Pick<PopoverContentProps, "align" | "side" | "sideOffset" | "collisionBoundary"> = {}) {
  const viewport = inject(appViewportKey);
  return computed(() => ({ collisionBoundary: viewport?.value ?? undefined, ...options }));
}
