import type { Ref } from "vue";
import { ApiCommandError } from "~/composables/useApi";

interface UseResourcePageErrorOptions {
  resourceName: string
  resourceId: Readonly<Ref<number | null>>
  resourceIsError: Readonly<Ref<boolean>>
  resourceError: Readonly<Ref<unknown>>
  fallbackErrorMessage: string
  invalidIdMessage?: string
  notFoundMessage?: string
}

const isApiNotFoundError = (error: unknown): boolean => {
  return error instanceof ApiCommandError && error.apiError === "NotFound";
};

export const useResourcePageError = ({
  resourceName,
  resourceId,
  resourceIsError,
  resourceError,
  fallbackErrorMessage,
  invalidIdMessage,
  notFoundMessage
}: UseResourcePageErrorOptions) => {
  const resourceNameLower = resourceName.toLowerCase();
  const nuxtError = useError();

  watchEffect(() => {
    if (nuxtError.value) return;

    if (resourceId.value === null) {
      showError(createError({
        status: 404,
        statusText: `${resourceName} not found`,
        message: invalidIdMessage ?? `The ${resourceNameLower} ID is missing or invalid.`,
        data: {
          source: "resource_page",
          resource: resourceNameLower,
          reason: "invalid_id"
        },
        fatal: true
      }));
      return;
    }

    if (!resourceIsError.value) return;
    if (isApiNotFoundError(resourceError.value)) {
      showError(createError({
        status: 404,
        statusText: `${resourceName} not found`,
        message: notFoundMessage ?? `The ${resourceNameLower} with ID ${resourceId.value} does not exist.`,
        data: {
          source: "resource_page",
          resource: resourceNameLower,
          reason: "api_not_found"
        },
        fatal: true
      }));
      return;
    }

    const error = resourceError.value;
    showError(createError({
      status: 500,
      statusText: `Failed to load ${resourceName}`,
      message: error instanceof Error && error.message.length
        ? error.message
        : fallbackErrorMessage,
      data: {
        source: "resource_page",
        resource: resourceNameLower,
        reason: "load_failed"
      },
      fatal: true
    }));
  });
};
