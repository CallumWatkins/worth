export default defineNuxtConfig({
  modules: [
    "@vueuse/nuxt",
    "@nuxt/ui",
    "reka-ui/nuxt",
    "@nuxt/eslint",
    "nuxt-echarts",
    "@nuxtjs/i18n"
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
  i18n: {
    defaultLocale: "en_gb",
    locales: ["en_gb"],
    strategy: "no_prefix"
  },
  ssr: false,
  devtools: { enabled: true },
  echarts: {
    renderer: ["canvas", "svg"],
    charts: [
      "LineChart",
      "PieChart"
    ],
    components: [
      "DatasetComponent",
      "GridComponent",
      "LegendComponent",
      "GraphicComponent",
      "TooltipComponent"
    ]
  },
  imports: {
    dirs: [
      "~/composables/**",
      "~/utils/**"
    ]
  },
  build: {
    transpile: ["echarts-liquidfill"]
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
    },
    resolve: {
      alias: { "echarts/lib/util/number": "echarts/lib/util/number.js" }
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
