import { commands } from "~/bindings";
import { invokeResult } from "~/utils/tauri-result";

const api = {
  hello: {
    say: (name: string) => invokeResult(commands.hello(name))
  }
};

export type Api = typeof api;

export const useApi = (): Api => {
  return api;
};
