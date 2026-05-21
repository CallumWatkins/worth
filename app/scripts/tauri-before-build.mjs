import { spawn } from "node:child_process";
import { access, stat } from "node:fs/promises";
import { createRequire } from "node:module";
import { join } from "node:path";

const require = createRequire(import.meta.url);

const posthogHost = "https://i.useworth.app";
const posthogReleaseName = "worth-desktop";
const publicOutputDir = join(process.cwd(), ".output", "public");
const bunCommand = process.versions.bun ? process.execPath : "bun";
const nodeCommand = process.platform === "win32" ? "node.exe" : "node";

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: "inherit",
      ...options
    });

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }

      const suffix = signal ? `signal ${signal}` : `exit code ${code}`;
      reject(new Error(`${command} ${args.join(" ")} failed with ${suffix}`));
    });
  });
}

function requiredEnv(name) {
  const value = process.env[name];
  if (value) return value;

  throw new Error(`${name} is required when POSTHOG_SOURCEMAPS=true`);
}

async function assertPublicOutputDir() {
  await access(publicOutputDir);

  const outputStat = await stat(publicOutputDir);
  if (!outputStat.isDirectory()) {
    throw new Error(`Expected ${publicOutputDir} to be a directory`);
  }
}

async function uploadPosthogSourcemaps() {
  if (process.env.POSTHOG_SOURCEMAPS !== "true") return;

  const releaseVersion = requiredEnv("APP_VERSION");
  // Avoid the extensionless .bin lookup that fails for PostHog CLI on Windows.
  const cliEntrypoint = require.resolve("@posthog/cli/run-posthog-cli.js");
  const cliEnv = {
    ...process.env,
    RUST_LOG: "posthog_cli=info",
    POSTHOG_CLI_HOST: posthogHost,
    POSTHOG_CLI_PROJECT_ID: requiredEnv("POSTHOG_PROJECT_ID"),
    POSTHOG_CLI_API_KEY: requiredEnv("POSTHOG_PERSONAL_API_KEY")
  };
  const commonArgs = [
    "--ignore",
    "**/node_modules/**",
    "--directory",
    publicOutputDir
  ];

  await assertPublicOutputDir();

  await runCommand(nodeCommand, [
    cliEntrypoint,
    "sourcemap",
    "inject",
    ...commonArgs,
    "--release-name",
    posthogReleaseName,
    "--release-version",
    releaseVersion
  ], { env: cliEnv });

  await runCommand(nodeCommand, [
    cliEntrypoint,
    "sourcemap",
    "upload",
    ...commonArgs,
    "--delete-after"
  ], { env: cliEnv });
}

async function run() {
  await runCommand(bunCommand, ["run", "generate"]);
  await uploadPosthogSourcemaps();
}

run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
