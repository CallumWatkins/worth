export default defineNuxtConfig({
  modules: [
    "@vueuse/nuxt",
    "@nuxt/ui",
    "reka-ui/nuxt",
    "@nuxt/eslint"
  ],
  app: {
    head: {
      title: "Worth",
      charset: "utf-8",
      viewport: "width=device-width, initial-scale=1",
      meta: [
        { name: "format-detection", content: "no" }
      ]
    },
    pageTransition: {
      name: "page",
      mode: "out-in"
    }
  },
  css: [
    "@/assets/css/main.css"
  ],
  ssr: false,
  devtools: { enabled: true },
  dir: {
    modules: "app/modules"
  },
  vite: {
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      strictPort: true,
      hmr: {
        protocol: "ws",
        host: "0.0.0.0",
        port: 3001
      }
    }
  },
  ignore: ["**/src-tauri/**"],
  devServer: {
    host: "0.0.0.0"
  },
  router: {
    options: {
      scrollBehaviorType: "smooth"
    }
  },
  eslint: {
    config: {
      standalone: false
    }
  },
  experimental: {
    typedPages: true
  },
  telemetry: false,
  compatibilityDate: "2026-01-01"
});
