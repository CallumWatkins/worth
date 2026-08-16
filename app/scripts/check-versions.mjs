import { readFile } from "node:fs/promises";

async function readKeyValueVersion(filePath) {
  const content = await readFile(filePath, "utf8");
  return content.match(/"version"\s*:\s*"([^"]+)"/)?.[1];
}

async function readCargoVersion(filePath) {
  const cargoToml = await readFile(filePath, "utf8");

  let inPackageSection = false;
  for (const line of cargoToml.split(/\r?\n/)) {
    const trimmedLine = line.trim();

    if (trimmedLine === "[package]") {
      inPackageSection = true;
      continue;
    }

    if (inPackageSection && trimmedLine.startsWith("[")) break;

    const version = trimmedLine.match(/^version\s*=\s*"([^"]+)"/)?.[1];
    if (inPackageSection && version) return version;
  }

  return undefined;
}

const versions = [
  ["package.json", await readKeyValueVersion("package.json")],
  ["src-tauri/tauri.conf.json", await readKeyValueVersion("src-tauri/tauri.conf.json")],
  ["src-tauri/Cargo.toml", await readCargoVersion("src-tauri/Cargo.toml")]
];

const expectedVersion = process.argv[2] ?? versions[0][1];
let failed = false;

for (const [filePath, version] of versions) {
  if (version === expectedVersion) continue;

  console.error(`${filePath}: expected version ${expectedVersion}, found ${version ?? "missing"}.`);
  failed = true;
}

if (failed) {
  process.exitCode = 1;
} else {
  console.log(`Source versions match ${expectedVersion}.`);
}
