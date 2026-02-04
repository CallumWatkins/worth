import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";

export default defineNuxtPlugin((nuxtApp) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        gcTime: 5 * 60_000,
        retry: 1
      }
    }
  });

  nuxtApp.vueApp.use(VueQueryPlugin, { queryClient });
});
