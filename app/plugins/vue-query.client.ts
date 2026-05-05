import { MutationCache, QueryCache, QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { ApiCommandError } from "~/composables/useApi";
import { reportHandledError } from "~/utils/error-reporting";

export default defineNuxtPlugin({
  name: "vue-query",
  setup(nuxtApp) {
    const queryClient = new QueryClient({
      queryCache: new QueryCache({
        onError: (error, query) => {
          if (error instanceof ApiCommandError) return;

          reportHandledError(error, {
            source: "vue_query",
            query_key: getRedactedQueryKey(query.queryKey)
          });
        }
      }),
      mutationCache: new MutationCache({
        onError: (error, _variables, _onMutateResult, mutation) => {
          if (error instanceof ApiCommandError) return;

          reportHandledError(error, {
            source: "vue_mutation",
            query_key: mutation.options.mutationKey === undefined
              ? undefined
              : getRedactedQueryKey(mutation.options.mutationKey)
          });
        }
      }),
      defaultOptions: {
        queries: {
          staleTime: 30_000,
          gcTime: 5 * 60_000,
          retry: false
        }
      }
    });

    nuxtApp.vueApp.use(VueQueryPlugin, { queryClient });
  }
});
