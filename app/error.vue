<template>
  <AppShell>
    <UMain class="error-page-enter flex flex-col">
      <UError
        :key="`${error.status}:${error.statusText}`"
        :error="error"
        :clear="false"
        :ui="{ root: 'min-h-0 flex-1 py-8' }"
      >
        <template #links>
          <UButton
            size="lg"
            @click="clearError({ redirect: '/' })"
          >
            Back to Dashboard
          </UButton>
          <UButton
            v-if="hasErrorDetailsSurvey"
            size="lg"
            color="neutral"
            variant="ghost"
            icon="i-lucide-message-circle-warning"
            @click="openErrorDetailsSurvey"
          >
            Send details
          </UButton>
        </template>
      </UError>
    </UMain>
  </AppShell>
</template>

<script setup lang="ts">
import type { NuxtError } from "#app";
import { reportHandledError } from "~/utils/error-reporting";

const props = defineProps<{
  error: NuxtError
}>();

const { hasErrorDetailsSurvey, openErrorDetailsSurvey } = useErrorDetailsSurvey();

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
