<template>
  <div
    data-tauri-drag-region="deep"
    class="pointer-events-auto fixed inset-x-0 top-0 z-50 flex h-(--app-titlebar-height) items-center bg-default"
    :class="{ 'pl-[80px]': isMac, 'text-muted': focused, 'text-dimmed': !focused }"
    role="group"
    aria-label="Window title bar"
    @pointerdown.stop
    @focusin.stop
  >
    <div class="flex shrink-0 items-center px-2">
      <UTooltip text="Back" :kbds="['alt', 'arrowleft']">
        <UButton
          icon="i-lucide-arrow-left"
          aria-label="Back"
          tabindex="-1"
          color="neutral"
          variant="ghost"
          size="xs"
          class="size-7 justify-center text-muted disabled:cursor-default"
          :class="backAvailable ? 'disabled:opacity-100' : 'disabled:opacity-30'"
          :disabled="!canGoBack"
          @click="back"
        />
      </UTooltip>
      <UTooltip text="Forward" :kbds="['alt', 'arrowright']">
        <UButton
          icon="i-lucide-arrow-right"
          aria-label="Forward"
          tabindex="-1"
          color="neutral"
          variant="ghost"
          size="xs"
          class="size-7 justify-center text-muted disabled:cursor-default"
          :class="forwardAvailable ? 'disabled:opacity-100' : 'disabled:opacity-30'"
          :disabled="!canGoForward"
          @click="forward"
        />
      </UTooltip>
    </div>
    <div class="flex h-full min-w-0 flex-1 items-center" @contextmenu="showWindowMenu">
      <span class="truncate text-xs">Worth</span>
    </div>
    <div v-if="!isMac" class="flex h-full shrink-0" role="group" aria-label="Window controls">
      <UButton
        square
        aria-label="Minimize window"
        tabindex="-1"
        color="neutral"
        variant="ghost"
        class="w-[46px] cursor-default justify-center rounded-none hover:text-highlighted"
        @click="appWindow?.minimize()"
      >
        <span v-if="isWindows" aria-hidden="true" class="windows-caption-glyph translate-y-[0.5px]">&#xE921;</span>
        <svg v-else aria-hidden="true" focusable="false" viewBox="0 0 10 10" class="size-2.5" fill="none" stroke="currentColor">
          <path d="M0 5.5h10" />
        </svg>
      </UButton>
      <UButton
        id="window-maximize"
        square
        :aria-label="maximized ? 'Restore window' : 'Maximize window'"
        tabindex="-1"
        color="neutral"
        variant="ghost"
        class="w-[46px] cursor-default justify-center rounded-none hover:text-highlighted"
        :class="{ 'bg-elevated text-highlighted': snap.hovered, 'bg-accented': snap.hovered && snap.pressed }"
        @click="appWindow?.toggleMaximize()"
      >
        <span v-if="isWindows" aria-hidden="true" class="windows-caption-glyph">{{ maximized ? "\uE923" : "\uE922" }}</span>
        <svg v-else aria-hidden="true" focusable="false" viewBox="0 0 10 10" class="size-2.5" fill="none" stroke="currentColor">
          <path v-if="maximized" d="M2.5 2.5v-2h7v7h-2M.5 2.5h7v7h-7z" />
          <rect v-else x=".5" y=".5" width="9" height="9" />
        </svg>
      </UButton>
      <UButton
        square
        aria-label="Close window"
        tabindex="-1"
        color="neutral"
        variant="ghost"
        class="w-[46px] cursor-default justify-center rounded-none hover:bg-red-600 hover:text-white"
        @click="appWindow?.close()"
      >
        <span v-if="isWindows" aria-hidden="true" class="windows-caption-glyph">&#xE8BB;</span>
        <svg v-else aria-hidden="true" focusable="false" viewBox="0 0 10 10" class="size-2.5" fill="none" stroke="currentColor">
          <path d="m.5.5 9 9m0-9-9 9" />
        </svg>
      </UButton>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const { backAvailable, forwardAvailable, canGoBack, canGoForward, back, forward } = useHistoryNavigation();
const isMac = navigator.userAgent.includes("Macintosh");
const isWindows = navigator.userAgent.includes("Windows");
const appWindow = isTauri() ? getCurrentWindow() : null;
const snap = useWindowSnap("window-maximize");
const focused = ref(true);
const maximized = ref(false);
const listeners: UnlistenFn[] = [];
let disposed = false;

function showWindowMenu(event: MouseEvent) {
  if (!isWindows || !appWindow) return;
  event.preventDefault();
  void invoke("plugin:window-snap|show_system_menu");
}

async function registerListener(registration: Promise<UnlistenFn>) {
  const unlisten = await registration;
  if (disposed) unlisten();
  else listeners.push(unlisten);
}

onMounted(async () => {
  if (!appWindow) return;
  await registerListener(appWindow.onFocusChanged(({ payload }) => {
    focused.value = payload;
  }));
  focused.value = await appWindow.isFocused();
  if (!isMac) {
    await registerListener(appWindow.onResized(() => {
      void appWindow.isMaximized().then((value) => {
        maximized.value = value;
      });
    }));
    maximized.value = await appWindow.isMaximized();
  }
});

onBeforeUnmount(() => {
  disposed = true;
  listeners.forEach((unlisten) => unlisten());
});
</script>

<style scoped>
.windows-caption-glyph {
  font-family: "Segoe Fluent Icons", "Segoe MDL2 Assets";
  font-size: 10px;
  font-weight: 400;
  line-height: 1;
}
</style>
