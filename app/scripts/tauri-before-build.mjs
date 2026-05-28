import { spawn } from "node:child_process";
import { access, readdir, rm, stat } from "node:fs/promises";
import { createRequire } from "node:module";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);

const rootDir = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const posthogHost = "https://i.useworth.app";
const posthogReleaseName = "worth-desktop";
const publicOutputDir = join(rootDir, ".output", "public");
const bunCommand = process.versions.bun ? process.execPath : "bun";
const nodeCommand = process.platform === "win32" ? "node.exe" : "node";

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: "inherit",
      cwd: rootDir,
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

async function collectFiles(directoryPath, predicate) {
  let entries;
  try {
    entries = await readdir(directoryPath, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") return [];
    throw error;
  }

  const files = [];
  for (const entry of entries) {
    const entryPath = join(directoryPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectFiles(entryPath, predicate));
    } else if (entry.isFile() && predicate(entryPath)) {
      files.push(entryPath);
    }
  }

  return files;
}

async function removeFrontendSourceMaps() {
  const sourceMapPaths = await collectFiles(join(publicOutputDir, "_nuxt"), (entryPath) => entryPath.endsWith(".map"));
  await Promise.all(sourceMapPaths.map((sourceMapPath) => rm(sourceMapPath, { force: true })));

  if (sourceMapPaths.length > 0) {
    console.log(`Deleted ${sourceMapPaths.length} frontend source map${sourceMapPaths.length === 1 ? "" : "s"} before packaging.`);
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
  // License generation and PostHog upload need source maps; packaged app assets must not include them.
  await runCommand(bunCommand, ["run", "app/scripts/generate-licenses.mjs"]);
  await uploadPosthogSourcemaps();
  await removeFrontendSourceMaps();
}

run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
