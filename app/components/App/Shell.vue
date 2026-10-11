<template>
  <Html :lang="lang" :dir="dir" class="select-none">
    <Body class="font-sans antialiased">
      <UTheme :props="popupDefaults">
        <UApp
          :locale="uiLocale"
          portal="#app-viewport"
          :scroll-body="{ padding: false, margin: false }"
        >
          <AppTitlebar />
          <div id="app-viewport" ref="viewport">
            <div id="app-scroll">
              <div class="app-page">
                <AppNavbar />
                <slot />
              </div>
            </div>
          </div>
        </UApp>
      </UTheme>
    </Body>
  </Html>
</template>

<script setup lang="ts">
import { useAppLocale } from "~/composables/useAppLocale";
import { appViewportKey } from "~/composables/useAppPopupContent";

const { dir, lang, uiLocale } = useAppLocale();
const viewport = useTemplateRef<HTMLElement>("viewport");
provide(appViewportKey, viewport);
const popupDefaults = computed(() => {
  const popup = { content: { collisionBoundary: viewport.value ?? undefined } };
  return {
    select: popup,
    selectMenu: popup,
    inputMenu: popup,
    dropdownMenu: popup,
    contextMenu: popup,
    popover: popup,
    tooltip: popup
  };
});
</script>
