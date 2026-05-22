import { spawn } from "node:child_process";
import { defineConfig } from "bumpp";

export default defineConfig({
  release: "prompt",
  commit: false,
  tag: false,
  push: false,
  files: [
    "package.json",
    "src-tauri/tauri.conf.json",
    "src-tauri/Cargo.toml"
  ],
  execute: ({ state }) => new Promise<void>((resolve, reject) => {
    const child = spawn("cargo", [
      "update",
      "--offline",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--package",
      "worth",
      "--precise",
      state.newVersion
    ], { stdio: "inherit" });

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(new Error(signal
        ? `cargo update was terminated by ${signal}`
        : `cargo update exited with code ${code}`));
    });
  })
});
