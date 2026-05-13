import type { ComputedRef, Ref } from "vue";

export interface KeyboardShortcutDefinition {
  label: string
  combos: string[][]
  handler?: () => void | Promise<void>
  usingInput?: boolean | string
  enabledWhenShortcutsOpen?: boolean
  order?: number
}

export interface KeyboardShortcutDisplayItem {
  label: string
  combos: string[][]
  order: number
}

export interface KeyboardShortcutsState {
  keyboardShortcutsOpen: Ref<boolean>
  shortcuts: ComputedRef<KeyboardShortcutDisplayItem[]>
}

let nextKeyboardShortcutSourceId = 0;

function getKeyboardShortcutSources() {
  return useState<Record<string, KeyboardShortcutDisplayItem[]>>("keyboardShortcutSources", () => ({}));
}

function getKeyboardShortcutName(combo: string[]) {
  return combo.map((key) => key.toLowerCase()).join("_");
}

export function useGlobalKeyboardShortcuts(definitions: (state: KeyboardShortcutsState) => KeyboardShortcutDefinition[]): KeyboardShortcutsState {
  const keyboardShortcutsOpen = useState("keyboardShortcutsOpen", () => false);
  const sources = getKeyboardShortcutSources();
  const shortcuts = computed(() => {
    return Object.values(sources.value)
      .flat()
      .sort((left, right) => left.order - right.order);
  });

  const state = {
    keyboardShortcutsOpen,
    shortcuts
  };

  registerKeyboardShortcuts(
    `shortcuts:${++nextKeyboardShortcutSourceId}`,
    typeof definitions === "function" ? definitions(state) : definitions
  );

  return state;
}

function registerKeyboardShortcuts(source: string, definitions: KeyboardShortcutDefinition[]) {
  const sources = getKeyboardShortcutSources();
  const keyboardShortcutsOpen = useState("keyboardShortcutsOpen", () => false);
  sources.value = {
    ...sources.value,
    [source]: definitions.map(({ label, combos, order = 0 }) => ({ label, combos, order }))
  };

  const shortcutConfig = Object.fromEntries(
    definitions
      .filter((shortcut) => shortcut.handler != null)
      .flatMap((shortcut) => shortcut.combos.map((combo) => {
        const config: { handler: () => void | Promise<void>, usingInput?: boolean | string } = {
          handler: async () => {
            if (keyboardShortcutsOpen.value && !shortcut.enabledWhenShortcutsOpen) return;
            await shortcut.handler!();
          }
        };

        if (shortcut.usingInput != null) {
          config.usingInput = shortcut.usingInput;
        }

        return [getKeyboardShortcutName(combo), config];
      }))
  );

  defineShortcuts(shortcutConfig);

  onUnmounted(() => {
    const { [source]: _removed, ...rest } = sources.value;
    sources.value = rest;
  });
}

export function useContextualKeyboardShortcuts(definitions: KeyboardShortcutDefinition[]) {
  useGlobalKeyboardShortcuts(() => definitions.map((shortcut) => ({
    ...shortcut,
    order: shortcut.order ?? 100
  })));
}
