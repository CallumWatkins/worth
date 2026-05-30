import type { Nuxt } from "@nuxt/schema";
import { APP_I18N_CONFIG } from "./app/utils/i18n";

const addUiIconsToIconClientBundle = (_options: unknown, nuxt: Nuxt) => {
  nuxt.hook("icon:clientBundleIcons", (icons) => {
    const uiIcons = nuxt.options.appConfig.ui?.icons as Record<string, unknown> | undefined;

    for (const icon of Object.values(uiIcons ?? {})) {
      if (typeof icon === "string") icons.add(icon.replace(/^i-([a-z0-9]+)-/, "$1:"));
    }
  });
};

export default defineNuxtConfig({
  modules: [
    "@vueuse/nuxt",
    "@nuxt/ui",
    addUiIconsToIconClientBundle,
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
    "@fontsource-variable/inter/wght.css",
    "@fontsource-variable/inter/wght-italic.css",
    "@/assets/css/main.css"
  ],
  ui: {
    fonts: false
  },
  icon: {
    provider: "none",
    fallbackToApi: false,
    clientBundle: {
      scan: {
        globInclude: [
          "app/**/*.{vue,ts}"
        ]
      }
    }
  },
  i18n: APP_I18N_CONFIG,
  ssr: false,
  typescript: {
    tsConfig: {
      compilerOptions: {
        target: "ES2025",
        lib: ["ES2025", "DOM", "WebWorker"]
      }
    }
  },
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
    build: {
      chunkSizeWarningLimit: 1500
    },
    optimizeDeps: {
      include: [
        "@internationalized/date",
        "@number-flow/vue",
        "@tanstack/vue-query",
        "@tanstack/vue-table",
        "@tauri-apps/api/app",
        "@tauri-apps/api/core",
        "@tauri-apps/api/event",
        "@tauri-apps/api/webviewWindow",
        "@tauri-apps/plugin-opener",
        "@vue/devtools-core",
        "@vue/devtools-kit",
        "posthog-js",
        "zod"
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
      alias: {
        "echarts/lib/util/number": "echarts/lib/util/number.js",
        "posthog-js": "posthog-js/dist/module.full.no-external"
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
  routeRules: process.env.NODE_ENV === "development"
    ? {
      "/**": {
        headers: {
          "Content-Security-Policy": [
            // CSP for dev server. For builds, CSP is configured in tauri.conf.json.
            // Dev-only allowances: localhost:3000 for Nuxt assets, localhost:3001/ws for Vite HMR,
            // and unsafe-inline/unsafe-eval for Nuxt/Vite dev scripts.
            "default-src 'self' http://localhost:3000",
            "connect-src 'self' ipc: http://ipc.localhost https://i.useworth.app http://localhost:3000 http://localhost:3001 ws:",
            "script-src 'self' http://localhost:3000 'unsafe-inline' 'unsafe-eval'",
            "style-src 'self' 'unsafe-inline' http://localhost:3000",
            "img-src 'self' data: blob: http://localhost:3000",
            "font-src 'self' data: http://localhost:3000",
            "object-src 'none'",
            "base-uri 'none'",
            "frame-src 'none'",
            "frame-ancestors 'none'",
            "form-action 'none'"
          ].join("; ")
        }
      }
    }
    : {},
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
      debug: process.env.NODE_ENV === "development",
      capture_exceptions: true,
      autocapture: false,
      capture_pageview: false,
      capture_pageleave: false,
      capture_dead_clicks: false,
      rageclick: false,
      disable_session_recording: true,
      disable_conversations: true,
      disable_product_tours: true,
      enable_recording_console_log: false,
      enable_heatmaps: false,
      disable_external_dependency_loading: true,
      person_profiles: "identified_only",
      mask_all_text: true,
      mask_all_element_attributes: true,
      advanced_disable_feature_flags: true,
      advanced_disable_feature_flags_on_first_load: true,
      internal_or_test_user_hostname: process.env.NODE_ENV === "development" ? "localhost" : null
    }
  }
});
