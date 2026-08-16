import type { MaybeRefOrGetter } from "vue";
import { toValue } from "vue";

export function useBlockRouteNavigationWhile(condition: MaybeRefOrGetter<boolean>) {
  const canNavigate = () => !toValue(condition);

  onBeforeRouteLeave(canNavigate);
  onBeforeRouteUpdate(canNavigate);
}
