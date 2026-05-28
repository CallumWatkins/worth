import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { constants as fsConstants } from "node:fs";
import { access, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { basename, dirname, isAbsolute, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const require = createRequire(import.meta.url);
const parseSpdxExpression = require("spdx-expression-parse");
const spdxLicenseList = require("spdx-license-list/full");

const rootDir = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const nodeModulesDir = join(rootDir, "node_modules");
const publicOutputDir = join(rootDir, ".output", "public");
const productionOutputPath = join(publicOutputDir, "licenses", "notices.json.gz");
const previewOutputPath = join(rootDir, "public", "licenses", "notices.json.gz");
const policyPath = join(rootDir, "license-policy.json");
const cargoManifestPath = join(rootDir, "src-tauri", "Cargo.toml");
const rootPackageJsonPath = join(rootDir, "package.json");
const ownLicensePath = join(rootDir, "LICENSE");
const cargoCommand = process.env.CARGO ?? "cargo";
const rustcCommand = process.env.RUSTC ?? "rustc";
const packageJsonExistsByPackageDir = new Map();

const ecosystemLabels = {
  frontend: "Frontend packages",
  rust: "Rust crates",
  fonts: "Bundled fonts"
};
const licenseFileNamePattern = /^(?:licen[cs]e|copying|notice|copyright|ofl|unlicense)(?:$|[._-])/i;
const missingLicensePattern = /^(?:unknown|noassertion|none)$/i;
const brandNotice = "The Worth name, logo, app icons, wordmarks, combination marks, source artwork, exported images, and copied brand assets are not licensed under AGPL-3.0-only. Copyright (c) 2026 Callum Watkins. All rights reserved except where explicit permission is granted.";

function printHelp() {
  console.log(`Generate Worth desktop license notices.

Usage:
  bun run app/scripts/generate-licenses.mjs [options]

Options:
  --preview                    Write public/licenses/notices.json.gz for tauri:dev.
  --output <path>              Write notices to a custom path.
  --cargo-target <target>      Resolve Rust dependencies for a target triple. Can be repeated.
  --cargo-targets <targets>    Comma-separated target triples.
  --target <target>            Alias for --cargo-target.
  --help                       Show this help.

The WORTH_LICENSE_CARGO_TARGETS environment variable may also contain a comma-separated target set.
Worth packaged builds are expected to be native; target options are for release/preflight dependency resolution.
`);
}

function parseArgs(argv) {
  const options = {
    preview: false,
    output: null,
    cargoTargets: []
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === "--help") {
      printHelp();
      process.exit(0);
    } else if (arg === "--preview") {
      options.preview = true;
    } else if (arg === "--output") {
      options.output = requireValue(argv, index, arg);
      index += 1;
    } else if (arg.startsWith("--output=")) {
      options.output = arg.slice("--output=".length);
    } else if (["--cargo-target", "--cargo-targets", "--target"].includes(arg)) {
      options.cargoTargets.push(...splitTargetSet(requireValue(argv, index, arg)));
      index += 1;
    } else if (arg.startsWith("--cargo-target=")) {
      options.cargoTargets.push(...splitTargetSet(arg.slice("--cargo-target=".length)));
    } else if (arg.startsWith("--cargo-targets=")) {
      options.cargoTargets.push(...splitTargetSet(arg.slice("--cargo-targets=".length)));
    } else if (arg.startsWith("--target=")) {
      options.cargoTargets.push(...splitTargetSet(arg.slice("--target=".length)));
    } else {
      throw new Error(`Unknown option: ${arg}`);
    }
  }

  return options;
}

function requireValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }

  return value;
}

function splitTargetSet(value) {
  return value
    .split(",")
    .map((target) => target.trim())
    .filter(Boolean);
}

function normalizeRequestedCargoTargets(cliTargets) {
  const envTargets = splitTargetSet(process.env.WORTH_LICENSE_CARGO_TARGETS ?? "");
  const requestedTargets = cliTargets.length > 0 ? cliTargets : envTargets;
  const normalizedTargets = requestedTargets.flatMap((target) => {
    if (target === "universal-apple-darwin") {
      return ["x86_64-apple-darwin", "aarch64-apple-darwin"];
    }

    return [target];
  });

  return [...new Set(normalizedTargets)];
}

async function resolveCargoTargets(cliTargets) {
  const requestedTargets = normalizeRequestedCargoTargets(cliTargets);
  if (requestedTargets.length > 0) return requestedTargets;

  const rustcVersion = await runCommand(rustcCommand, ["-vV"]);
  const hostTarget = rustcVersion.match(/^host: (.+)$/m)?.[1];
  if (!hostTarget) throw new Error("Could not determine the local Rust host target from rustc -vV.");

  return [hostTarget];
}

async function readJsonFile(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

async function pathExists(filePath) {
  try {
    await access(filePath, fsConstants.F_OK);
    return true;
  } catch {
    return false;
  }
}

async function packageJsonExists(packageDir) {
  if (!packageJsonExistsByPackageDir.has(packageDir)) {
    packageJsonExistsByPackageDir.set(packageDir, await pathExists(join(packageDir, "package.json")));
  }

  return packageJsonExistsByPackageDir.get(packageDir);
}

function runCommand(command, args) {
  return new Promise((resolveCommand, reject) => {
    const child = spawn(command, args, {
      cwd: rootDir,
      stdio: ["ignore", "pipe", "inherit"]
    });
    let stdout = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (code === 0) {
        resolveCommand(stdout);
        return;
      }

      const suffix = signal ? `signal ${signal}` : `exit code ${code}`;
      reject(new Error(`${command} ${args.join(" ")} failed with ${suffix}`));
    });
  });
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
    } else if (entry.isFile() && predicate(entryPath, entry)) {
      files.push(entryPath);
    }
  }

  return files;
}

function decodeSource(source) {
  try {
    return decodeURIComponent(source);
  } catch {
    return source;
  }
}

function isKnownVirtualSource(source) {
  const normalizedSource = source.replace(/\\/g, "/").toLowerCase();
  return normalizedSource.startsWith("\0")
    || normalizedSource.startsWith("virtual:")
    || normalizedSource.includes("/virtual:")
    || normalizedSource.includes("virtual:nuxt:")
    || normalizedSource.includes("/@vite/")
    || normalizedSource.includes("/node_modules/.cache/")
    || normalizedSource.includes("/node_modules/.vite/");
}

function stripSourcePart(part) {
  return part.split(/[?#]/, 1)[0];
}

function pathFromParts(parts) {
  const prefix = parts[0] === "" ? sep : "";
  return `${prefix}${parts.filter(Boolean).join(sep)}`;
}

function packageInfoFromPath(sourcePath) {
  const parts = sourcePath.replace(/\\/g, "/").split("/").map(stripSourcePart);
  let nodeModulesIndex = -1;
  let packagePartCount = 0;

  for (let index = 0; index < parts.length; index += 1) {
    if (parts[index] !== "node_modules") continue;

    const firstPackagePart = parts[index + 1];
    if (!firstPackagePart || firstPackagePart.startsWith(".")) continue;

    nodeModulesIndex = index;
    packagePartCount = firstPackagePart.startsWith("@") ? 2 : 1;
  }

  if (nodeModulesIndex === -1) return null;

  const packageEndIndex = nodeModulesIndex + 1 + packagePartCount;
  if (parts.length <= packageEndIndex - 1) return null;

  return {
    packageName: parts.slice(nodeModulesIndex + 1, packageEndIndex).join("/"),
    packageDir: pathFromParts(parts.slice(0, packageEndIndex))
  };
}

function sourcePathFromSource(source, sourceMapPath) {
  if (source.startsWith("file:")) {
    try {
      return fileURLToPath(source);
    } catch {
      // Fall through to path handling for malformed file URLs from sourcemaps.
    }
  }

  const normalizedSource = source.replace(/\\/g, "/");
  const withoutDotPrefix = normalizedSource.replace(/^\.\//, "");
  if (withoutDotPrefix.startsWith("node_modules/")) return resolve(rootDir, withoutDotPrefix);

  if (isAbsolute(source) || /^[a-z]:[\\/]/i.test(source)) return source;

  return resolve(dirname(sourceMapPath), source);
}

async function nodePackageDirFromSource(rawSource, sourceMapPath) {
  const source = decodeSource(rawSource);
  const normalizedSource = source.replace(/\\/g, "/");

  if (!normalizedSource.includes("node_modules")) return null;
  if (isKnownVirtualSource(normalizedSource)) return { ignored: true };

  const sourcePath = sourcePathFromSource(source, sourceMapPath);
  const sourcePathPackageInfo = packageInfoFromPath(sourcePath);
  const rawSourcePackageInfo = packageInfoFromPath(source);
  const packageName = sourcePathPackageInfo?.packageName ?? rawSourcePackageInfo?.packageName;

  if (!packageName) return { unmapped: rawSource };

  if (sourcePathPackageInfo?.packageDir && await packageJsonExists(sourcePathPackageInfo.packageDir)) {
    return { packageDir: sourcePathPackageInfo.packageDir };
  }

  return { packageDir: rootNodePackageDir(packageName) };
}

function isFontPackageName(packageName) {
  return packageName.startsWith("@fontsource/") || packageName.startsWith("@fontsource-variable/");
}

function rootNodePackageDir(packageName) {
  return join(nodeModulesDir, ...packageName.split("/"));
}

async function collectFrontendPackageDirs(warnings) {
  if (!await pathExists(publicOutputDir)) {
    throw new Error(`Generated frontend output was not found at ${publicOutputDir}. Run bun run generate first.`);
  }

  const nuxtOutputDir = join(publicOutputDir, "_nuxt");
  const sourceMapPaths = await collectFiles(nuxtOutputDir, (entryPath) => entryPath.endsWith(".map"));
  if (sourceMapPaths.length === 0) {
    throw new Error(`No frontend source maps were found in ${nuxtOutputDir}; license generation cannot map bundled code to packages.`);
  }

  const packageDirs = new Set();
  const unmappedSources = new Set();

  for (const sourceMapPath of sourceMapPaths) {
    let sourceMap;
    try {
      sourceMap = JSON.parse(await readFile(sourceMapPath, "utf8"));
    } catch (error) {
      warnings.push(`Could not read source map ${sourceMapPath}: ${error.message}`);
      continue;
    }

    if (!Array.isArray(sourceMap.sources)) continue;

    for (const source of sourceMap.sources) {
      if (typeof source !== "string") continue;

      const result = await nodePackageDirFromSource(source, sourceMapPath);
      if (!result || result.ignored) continue;
      if (result.packageDir) {
        packageDirs.add(result.packageDir);
      } else if (result.unmapped) {
        unmappedSources.add(result.unmapped);
      }
    }
  }

  for (const source of unmappedSources) {
    warnings.push(`Bundled frontend source from node_modules could not be mapped to a package: ${source}`);
  }

  await addBundledFontPackageDirs(packageDirs);

  return packageDirs;
}

async function addBundledFontPackageDirs(packageDirs) {
  const packageJson = await readJsonFile(rootPackageJsonPath);
  const dependencyNames = Object.keys(packageJson.dependencies ?? {});
  const fontDependencyNames = dependencyNames.filter(isFontPackageName);
  if (fontDependencyNames.length === 0) return;

  const bundledFontFiles = await collectFiles(publicOutputDir, (entryPath) => /\.(?:woff2?|ttf|otf)$/i.test(entryPath));
  if (bundledFontFiles.length === 0) return;

  for (const packageName of fontDependencyNames) {
    packageDirs.add(rootNodePackageDir(packageName));
  }
}

async function collectFrontendPackages(policy, warnings) {
  const packageDirs = await collectFrontendPackageDirs(warnings);
  const packages = new Map();

  for (const packageDir of [...packageDirs].sort(compareStrings)) {
    const packageJsonPath = join(packageDir, "package.json");
    let packageJson;
    try {
      packageJson = await readJsonFile(packageJsonPath);
    } catch (error) {
      warnings.push(`Could not read package metadata at ${packageJsonPath}: ${error.message}`);
      continue;
    }

    const ecosystem = isFontPackageName(packageJson.name) ? "fonts" : "frontend";
    const noticePackage = await createPackageNotice({
      ecosystem,
      name: packageJson.name,
      version: packageJson.version ?? "0.0.0",
      license: normalizePackageJsonLicense(packageJson.license ?? packageJson.licenses),
      packageDir,
      explicitLicenseFiles: []
    }, policy, warnings);
    packages.set(packageKey(noticePackage), noticePackage);
  }

  return [...packages.values()];
}

function normalizePackageJsonLicense(value) {
  if (typeof value === "string") return value;

  if (Array.isArray(value)) {
    const licenses = value.map(normalizePackageJsonLicense).filter(Boolean);
    return licenses.length > 0 ? licenses.join(" OR ") : null;
  }

  if (value && typeof value === "object" && typeof value.type === "string") {
    return value.type;
  }

  return null;
}

async function cargoMetadata(target) {
  const args = [
    "metadata",
    "--manifest-path",
    cargoManifestPath,
    "--format-version",
    "1",
    "--locked"
  ];

  if (target) args.push("--filter-platform", target);

  console.log(`Resolving Rust dependencies for ${target}.`);

  return JSON.parse(await runCommand(cargoCommand, args));
}

function isNormalCargoDependency(dependency) {
  return dependency.dep_kinds.some((depKind) => (depKind.kind ?? "normal") === "normal");
}

function isProcMacroOnlyPackage(cargoPackage) {
  const targets = cargoPackage?.targets ?? [];
  return targets.length > 0 && targets.every((target) => target.kind?.includes("proc-macro"));
}

function reachableCargoPackageIds(metadata, packagesById) {
  const rootId = metadata.resolve?.root ?? metadata.workspace_members?.[0];
  if (!rootId) return new Set();

  const nodesById = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const reachable = new Set();
  const stack = [rootId];

  while (stack.length > 0) {
    const id = stack.pop();
    if (reachable.has(id)) continue;

    reachable.add(id);
    const node = nodesById.get(id);
    if (!node) continue;

    for (const dependency of node.deps.filter(isNormalCargoDependency)) {
      if (isProcMacroOnlyPackage(packagesById.get(dependency.pkg))) continue;

      stack.push(dependency.pkg);
    }
  }

  return reachable;
}

async function collectRustPackages(cargoTargets, policy, warnings) {
  const packages = new Map();

  for (const target of cargoTargets) {
    const metadata = await cargoMetadata(target);
    const packagesById = new Map(metadata.packages.map((cargoPackage) => [cargoPackage.id, cargoPackage]));
    const reachableIds = reachableCargoPackageIds(metadata, packagesById);

    for (const packageId of reachableIds) {
      const cargoPackage = packagesById.get(packageId);
      if (!cargoPackage || cargoPackage.source == null || isProcMacroOnlyPackage(cargoPackage)) continue;

      const noticePackage = await createPackageNotice({
        ecosystem: "rust",
        name: cargoPackage.name,
        version: cargoPackage.version,
        license: cargoPackage.license,
        packageDir: dirname(cargoPackage.manifest_path),
        explicitLicenseFiles: cargoPackage.license_file ? [cargoPackage.license_file] : []
      }, policy, warnings);
      packages.set(packageKey(noticePackage), noticePackage);
    }
  }

  return [...packages.values()];
}

function normalizeLicense(license) {
  if (typeof license !== "string") return null;

  const normalized = license.trim().replace(/\s+/g, " ");
  return normalized.length > 0 ? normalized : null;
}

function findPolicyEntry(entries, noticePackage) {
  return entries.find((entry) => {
    if (entry.ecosystem && entry.ecosystem !== noticePackage.ecosystem) return false;
    if (entry.name !== noticePackage.name) return false;
    if (entry.version && entry.version !== noticePackage.version) return false;
    if (entry.license && entry.license !== noticePackage.license) return false;
    return true;
  });
}

function findLicenseOverride(entries, noticePackage) {
  return entries.find((entry) => {
    if (entry.ecosystem && entry.ecosystem !== noticePackage.ecosystem) return false;
    if (entry.name !== noticePackage.name) return false;
    if (entry.version && entry.version !== noticePackage.version) return false;
    if (entry.matchLicense && entry.matchLicense !== noticePackage.license) return false;
    return true;
  });
}

async function createPackageNotice(input, policy, warnings) {
  const rawLicense = normalizeLicense(input.license);
  const draftPackage = {
    ecosystem: input.ecosystem,
    name: input.name,
    version: input.version,
    license: rawLicense && !isUnknownLicense(rawLicense) ? rawLicense : "Unknown"
  };
  const override = findLicenseOverride(policy.licenseOverrides ?? [], draftPackage);
  const overrideLicense = normalizeLicense(override?.license);
  const noticePackage = {
    ...draftPackage,
    license: overrideLicense && !isUnknownLicense(overrideLicense) ? overrideLicense : draftPackage.license
  };
  const texts = await collectPackageTexts({
    ...noticePackage,
    packageDir: input.packageDir,
    explicitLicenseFiles: input.explicitLicenseFiles
  }, warnings);

  return {
    ...noticePackage,
    texts
  };
}

function isNoticeFile(filePath) {
  return basename(filePath).toLowerCase().startsWith("notice");
}

async function candidateLicenseTextFiles(packageDir, explicitLicenseFiles) {
  const candidates = [];
  const seenPaths = new Set();

  for (const explicitLicenseFile of explicitLicenseFiles) {
    const resolvedPath = isAbsolute(explicitLicenseFile)
      ? explicitLicenseFile
      : join(packageDir, explicitLicenseFile);
    if (seenPaths.has(resolvedPath) || !await pathExists(resolvedPath)) continue;

    seenPaths.add(resolvedPath);
    candidates.push(resolvedPath);
  }

  let entries;
  try {
    entries = await readdir(packageDir, { withFileTypes: true });
  } catch {
    return candidates;
  }

  for (const entry of entries) {
    if (!entry.isFile() || !licenseFileNamePattern.test(entry.name)) continue;

    const filePath = join(packageDir, entry.name);
    if (seenPaths.has(filePath)) continue;

    seenPaths.add(filePath);
    candidates.push(filePath);
  }

  return candidates.sort((first, second) => {
    const firstIsNotice = isNoticeFile(first);
    const secondIsNotice = isNoticeFile(second);
    if (firstIsNotice !== secondIsNotice) return firstIsNotice ? 1 : -1;
    return compareStrings(basename(first), basename(second));
  });
}

async function readTextFile(filePath, noticePackage, warnings) {
  const fileStat = await stat(filePath);
  if (fileStat.size > 1_000_000) {
    warnings.push(`${noticePackage.name}@${noticePackage.version} has a large license file skipped from notices: ${filePath}`);
    return null;
  }

  const text = (await readFile(filePath, "utf8")).replace(/\r\n/g, "\n").trim();
  if (!text || text.includes("\u0000")) return null;

  return text;
}

function normalizeLicenseExpression(license) {
  return license.replace(/\s*\/\s*/g, " OR ").replace(/,/g, " OR ");
}

function parseLicenseExpression(license) {
  if (!license || isUnknownLicense(license)) {
    return null;
  }

  const normalizedLicense = normalizeLicenseExpression(license);
  try {
    return parseSpdxExpression(normalizedLicense);
  } catch (error) {
    throw new Error(`Invalid SPDX expression "${license}" after normalization to "${normalizedLicense}": ${error.message}`);
  }
}

function licenseExpressionId(expression) {
  return `${expression.license}${expression.plus ? "+" : ""}`;
}

function licenseExpressionDetails(expression) {
  if (!expression) return { licenses: [], baseLicenses: [], exceptions: [] };

  if (expression.license) {
    return {
      licenses: [licenseExpressionId(expression)],
      baseLicenses: [expression.license],
      exceptions: expression.exception ? [expression.exception] : []
    };
  }

  const details = [expression.left, expression.right].map(licenseExpressionDetails);
  return {
    licenses: [...new Set(details.flatMap((entry) => entry.licenses))],
    baseLicenses: [...new Set(details.flatMap((entry) => entry.baseLicenses))],
    exceptions: [...new Set(details.flatMap((entry) => entry.exceptions))]
  };
}

function isLicenseExpressionApproved(expression, approvedLicenses, approvedExceptions) {
  if (!expression) return false;
  if (expression.conjunction === "or") {
    return isLicenseExpressionApproved(expression.left, approvedLicenses, approvedExceptions)
      || isLicenseExpressionApproved(expression.right, approvedLicenses, approvedExceptions);
  }

  if (expression.conjunction === "and") {
    return isLicenseExpressionApproved(expression.left, approvedLicenses, approvedExceptions)
      && isLicenseExpressionApproved(expression.right, approvedLicenses, approvedExceptions);
  }

  return approvedLicenses.has(licenseExpressionId(expression))
    && (!expression.exception || approvedExceptions.has(expression.exception));
}

function standardLicenseTexts(license, noticePackage, warnings) {
  let details;
  try {
    details = licenseExpressionDetails(parseLicenseExpression(license));
  } catch (error) {
    warnings.push(`${noticePackage.name}@${noticePackage.version} uses ${license}, but it could not be parsed as SPDX for standard license text fallback: ${error.message}`);
    return [];
  }

  if (details.exceptions.length > 0) {
    warnings.push(`${noticePackage.name}@${noticePackage.version} uses SPDX exception${details.exceptions.length === 1 ? "" : "s"} ${details.exceptions.join(", ")}, but no bundled license file was found; standard fallback text includes only the base license text.`);
  }

  const texts = [];

  for (const licenseId of details.baseLicenses) {
    if (licenseId.startsWith("LicenseRef-")) continue;

    const licenseRecord = spdxLicenseList[licenseId];
    if (licenseRecord?.licenseText) {
      texts.push({
        title: `Standard ${licenseId} license text`,
        text: licenseRecord.licenseText.replace(/\r\n/g, "\n").trim()
      });
    } else {
      warnings.push(`${noticePackage.name}@${noticePackage.version} uses ${licenseId}, but no standard SPDX text was available.`);
    }
  }

  return texts;
}

function textHash(text) {
  return createHash("sha256").update(text).digest("hex");
}

async function collectPackageTexts(noticePackage, warnings) {
  const texts = [];
  const seenHashes = new Set();
  const candidateFiles = await candidateLicenseTextFiles(noticePackage.packageDir, noticePackage.explicitLicenseFiles);

  for (const filePath of candidateFiles) {
    let text;
    try {
      text = await readTextFile(filePath, noticePackage, warnings);
    } catch (error) {
      warnings.push(`Could not read license text for ${noticePackage.name}@${noticePackage.version} from ${filePath}: ${error.message}`);
      continue;
    }

    if (!text) continue;

    const hash = textHash(text);
    if (seenHashes.has(hash)) continue;

    seenHashes.add(hash);
    texts.push({
      title: basename(filePath),
      kind: isNoticeFile(filePath) ? "notice" : "license",
      text
    });
  }

  if (!texts.some((entry) => entry.kind === "license")) {
    for (const fallbackText of standardLicenseTexts(noticePackage.license, noticePackage, warnings)) {
      const hash = textHash(fallbackText.text);
      if (seenHashes.has(hash)) continue;

      seenHashes.add(hash);
      texts.unshift({
        ...fallbackText,
        kind: "license"
      });
    }
  }

  if (texts.length === 0) {
    warnings.push(`${noticePackage.name}@${noticePackage.version} has no bundled license or notice text.`);
    texts.push({
      title: "License text unavailable",
      kind: "notice",
      text: "No bundled license or notice text was found for this package, and no standard SPDX license text was available from its license metadata."
    });
  }

  return texts;
}

function isUnknownLicense(license) {
  if (!license || missingLicensePattern.test(license)) return true;

  const normalizedLicense = license.trim().toLowerCase();
  return normalizedLicense.startsWith("see license in ") || normalizedLicense.startsWith("see licence in ");
}

function policyFailures(packages, policy) {
  const approvedLicenses = new Set(policy.approvedLicenses ?? []);
  const approvedExceptions = new Set(policy.approvedExceptions ?? []);
  const failures = [];

  for (const noticePackage of packages) {
    if (isUnknownLicense(noticePackage.license)) {
      failures.push(`${ecosystemLabels[noticePackage.ecosystem]}: ${noticePackage.name}@${noticePackage.version} has unknown or missing license metadata.`);
      continue;
    }

    const reviewedPackage = findPolicyEntry(policy.reviewedPackages ?? [], noticePackage);
    let licenseExpression;
    try {
      licenseExpression = parseLicenseExpression(noticePackage.license);
    } catch (error) {
      const message = `${ecosystemLabels[noticePackage.ecosystem]}: ${noticePackage.name}@${noticePackage.version} has invalid license metadata ${noticePackage.license}: ${error.message}.`;
      if (reviewedPackage?.license === noticePackage.license) {
        continue;
      }

      failures.push(message);
      continue;
    }

    if (isLicenseExpressionApproved(licenseExpression, approvedLicenses, approvedExceptions)) continue;

    const { licenses, exceptions } = licenseExpressionDetails(licenseExpression);
    const unapprovedLicenses = licenses.filter((licenseId) => !approvedLicenses.has(licenseId));
    const unapprovedExceptions = exceptions.filter((exceptionId) => !approvedExceptions.has(exceptionId));
    if (reviewedPackage) continue;

    failures.push(`${ecosystemLabels[noticePackage.ecosystem]}: ${noticePackage.name}@${noticePackage.version} uses ${noticePackage.license}. Unapproved identifiers: ${[
      ...unapprovedLicenses,
      ...unapprovedExceptions
    ].join(", ") || "none"}.`);
  }

  return failures;
}

function packageKey(noticePackage) {
  return `${noticePackage.ecosystem}:${noticePackage.name}@${noticePackage.version}`;
}

function compareStrings(first, second) {
  return first.localeCompare(second, "en", { sensitivity: "base" });
}

function comparePackages(first, second) {
  const nameComparison = compareStrings(first.name, second.name);
  if (nameComparison !== 0) return nameComparison;

  const versionComparison = compareStrings(first.version, second.version);
  if (versionComparison !== 0) return versionComparison;

  return compareStrings(first.ecosystem, second.ecosystem);
}

async function buildNotices(packages) {
  const packageJson = await readJsonFile(rootPackageJsonPath);
  const licenseText = (await readFile(ownLicensePath, "utf8")).replace(/\r\n/g, "\n").trim();

  return {
    app: {
      name: "Worth",
      version: packageJson.version,
      license: packageJson.license,
      brandNotice,
      licenseText
    },
    thirdParty: packages
  };
}

function packageCountsByEcosystem(packages) {
  return Object.keys(ecosystemLabels)
    .map((ecosystem) => ({
      ecosystem,
      count: packages.filter((noticePackage) => noticePackage.ecosystem === ecosystem).length
    }))
    .filter((entry) => entry.count > 0);
}

function printFailures(failures) {
  for (const failure of [...new Set(failures)].sort(compareStrings)) {
    console.error(`license error: ${failure}`);
  }
}

async function run() {
  const options = parseArgs(process.argv.slice(2));
  const resolvedCargoTargets = await resolveCargoTargets(options.cargoTargets);
  const outputPath = resolve(rootDir, options.output ?? (options.preview ? previewOutputPath : productionOutputPath));
  const policy = await readJsonFile(policyPath);
  const collectedFailures = [];

  const [frontendPackages, rustPackages] = await Promise.all([
    collectFrontendPackages(policy, collectedFailures),
    collectRustPackages(resolvedCargoTargets, policy, collectedFailures)
  ]);
  const packages = [...frontendPackages, ...rustPackages].sort(comparePackages);
  const failures = [
    ...collectedFailures,
    ...policyFailures(packages, policy)
  ];

  if (failures.length > 0) {
    printFailures(failures);
    throw new Error("License policy check failed.");
  }

  const notices = await buildNotices(packages);
  const noticesJson = JSON.stringify(notices);
  const compressedNotices = gzipSync(noticesJson, { level: 9 });
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, compressedNotices);

  console.log(`Generated compressed license notices at ${outputPath}.`);
  console.log(`Compressed ${(noticesJson.length / 1024).toFixed(1)} KiB to ${(compressedNotices.byteLength / 1024).toFixed(1)} KiB.`);
  for (const { ecosystem, count } of packageCountsByEcosystem(notices.thirdParty)) {
    console.log(`Included ${count} ${ecosystemLabels[ecosystem].toLowerCase()}.`);
  }
}

run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
