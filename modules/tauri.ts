import * as tauriApp from "@tauri-apps/api/app";
import * as tauriWebviewWindow from "@tauri-apps/api/webviewWindow";
import { addImports, defineNuxtModule } from "nuxt/kit";

const capitalize = (name: string) => {
  return name.charAt(0).toUpperCase() + name.slice(1);
};

const tauriModules = [
  { module: tauriApp, prefix: "App", importPath: "@tauri-apps/api/app" },
  { module: tauriWebviewWindow, prefix: "WebviewWindow", importPath: "@tauri-apps/api/webviewWindow" }
];

export default defineNuxtModule({
  meta: {
    name: "nuxt-tauri",
    configKey: "tauri"
  },
  defaults: {
    prefix: "useTauri"
  },
  setup(options) {
    tauriModules.forEach(({ module, prefix, importPath }) => {
      Object.keys(module).filter((name) => name !== "default").forEach((name) => {
        const prefixedName = `${options.prefix}${prefix}` || "";
        const as = prefixedName ? prefixedName + capitalize(name) : name;
        addImports({ from: importPath, name, as });
      });
    });
  }
});
