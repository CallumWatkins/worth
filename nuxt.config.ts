import { APP_I18N_CONFIG } from "./app/utils/i18n";

export default defineNuxtConfig({
  modules: [
    "@vueuse/nuxt",
    "@nuxt/ui",
    "reka-ui/nuxt",
    "@nuxt/eslint",
    "nuxt-echarts",
    "@nuxtjs/i18n",
    "@posthog/nuxt"
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
  i18n: APP_I18N_CONFIG,
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
    optimizeDeps: {
      include: [
        "@vue/devtools-core",
        "@vue/devtools-kit",
        "@tanstack/vue-query",
        "@tauri-apps/api/core",
        "@tauri-apps/api/event",
        "@tauri-apps/api/webviewWindow",
        "@internationalized/date",
        "zod",
        "@tanstack/vue-table"
      ]
    },
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
  compatibilityDate: "2026-01-01",
  sourcemap: {
    client: "hidden"
  },
  posthogConfig: {
    publicKey: "phc_pREdKqdwjhVFDkWkFrTzzEruDgVM7vfxYDyyWCPFz737",
    host: "https://i.useworth.app",
    clientConfig: {
      defaults: "2026-01-30",
      capture_exceptions: true,
      autocapture: false,
      capture_pageview: false,
      capture_pageleave: false,
      capture_dead_clicks: false,
      rageclick: false,
      disable_session_recording: true,
      enable_recording_console_log: false,
      enable_heatmaps: false,
      disable_surveys: true,
      person_profiles: "identified_only",
      mask_all_text: true,
      mask_all_element_attributes: true,
      advanced_disable_feature_flags: true,
      advanced_disable_feature_flags_on_first_load: true,
      internal_or_test_user_hostname: process.env.NODE_ENV === "development" ? "localhost" : null
    },
    sourcemaps: {
      enabled: process.env.POSTHOG_SOURCEMAPS === "true",
      projectId: process.env.POSTHOG_PROJECT_ID!,
      personalApiKey: process.env.POSTHOG_PERSONAL_API_KEY!,
      releaseName: "worth-desktop",
      releaseVersion: process.env.APP_VERSION,
      deleteAfterUpload: true
    }
  }
});
