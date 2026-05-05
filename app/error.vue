<template>
  <Html :lang="lang" :dir="dir" class="overflow-x-hidden select-none">
    <Body class="font-sans antialiased">
      <UApp
        :locale="uiLocale"
        :scroll-body="{ padding: false, margin: false }"
      >
        <AppNavbar />
        <UMain class="error-page-enter">
          <UError
            :key="`${error.status}:${error.statusText}`"
            :error="error"
            redirect="/"
            :clear="{
              label: 'Back to Dashboard'
            }"
          />
        </UMain>
      </UApp>
    </Body>
  </Html>
</template>

<script setup lang="ts">
import type { NuxtError } from "#app";
import { useAppLocale } from "~/composables/useAppLocale";
import { reportHandledError } from "~/utils/error-reporting";

const props = defineProps<{
  error: NuxtError
}>();

const { dir, lang, uiLocale } = useAppLocale();

onMounted(() => {
  const status = props.error.status ?? props.error.statusCode ?? 500;
  const statusText = props.error.statusText ?? props.error.statusMessage ?? "Error";
  const error = new Error(`Nuxt error page rendered: ${status} ${statusText}`);
  error.name = "NuxtErrorPage";

  reportHandledError(error, {
    source: "nuxt_error_page",
    nuxt_error_status: status,
    nuxt_error_status_text: statusText,
    nuxt_error_fatal: props.error.fatal === true,
    ...getNuxtErrorDataProperties(props.error.data)
  });
});

function getNuxtErrorDataProperties(data: unknown) {
  if (typeof data !== "object" || data == null) return {};

  const record = data as Record<string, unknown>;
  const properties: Record<string, string> = {};
  if (typeof record.source === "string") properties.nuxt_error_source = record.source;
  if (typeof record.resource === "string") properties.nuxt_error_resource = record.resource;
  if (typeof record.reason === "string") properties.nuxt_error_reason = record.reason;
  return properties;
}
</script>

<style scoped>
.error-page-enter {
  animation: error-page-fade-in 200ms ease-in-out both;
}

@keyframes error-page-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
