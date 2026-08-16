import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";

const migrationsPath = "src-tauri/db/migrations";
const stableTagPattern = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

function runGit(args, { allowFailure = false } = {}) {
  const result = spawnSync("git", args, {
    encoding: "utf8"
  });

  if (result.error) throw result.error;
  if (!allowFailure && result.status !== 0) {
    process.stderr.write(result.stderr);
    process.exit(result.status ?? 1);
  }

  return result;
}

function parseArguments() {
  let baselineTag;
  let excludedTag;

  for (let index = 2; index < process.argv.length; index += 1) {
    const argument = process.argv[index];
    const value = process.argv[index + 1];

    if (argument === "--baseline-tag" && value) {
      baselineTag = value;
      index += 1;
    } else if (argument === "--exclude-tag" && value) {
      excludedTag = value;
      index += 1;
    } else {
      console.error(`Unknown or incomplete argument: ${argument}`);
      process.exit(2);
    }
  }

  if (baselineTag && excludedTag) {
    console.error("--baseline-tag and --exclude-tag cannot be used together.");
    process.exit(2);
  }

  return { baselineTag, excludedTag };
}

function stableTagVersion(tag) {
  const match = tag.match(stableTagPattern);
  return match?.slice(1).map(Number);
}

function compareVersions(left, right) {
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) return left[index] - right[index];
  }
  return 0;
}

function latestStableTag(excludedTag) {
  return runGit(["tag", "--list", "v*"])
    .stdout
    .split(/\r?\n/)
    .filter((tag) => tag && tag !== excludedTag)
    .map((tag) => [tag, stableTagVersion(tag)])
    .filter((entry) => entry[1] != null)
    .sort((left, right) => compareVersions(right[1], left[1]))[0]?.[0];
}

function nullSeparatedGitOutput(args) {
  return runGit(args)
    .stdout
    .split("\0")
    .filter(Boolean);
}

const { baselineTag: requestedBaselineTag, excludedTag } = parseArguments();
const baselineTag = requestedBaselineTag ?? latestStableTag(excludedTag);
const currentMigrationFiles = nullSeparatedGitOutput([
  "ls-files",
  "--cached",
  "--others",
  "--exclude-standard",
  "-z",
  "--",
  migrationsPath
]).filter((filePath) => filePath.endsWith(".sql"));

let failed = false;

for (const filePath of currentMigrationFiles) {
  const attributes = runGit(["check-attr", "eol", "--", filePath]).stdout.trim();
  if (!attributes.endsWith(": eol: lf")) {
    console.error(`${filePath}: expected Git attribute eol=lf.`);
    failed = true;
  }

  let contents;
  try {
    contents = await readFile(filePath);
  } catch (error) {
    if (error.code === "ENOENT") continue;
    throw error;
  }

  if (contents.includes(13)) {
    console.error(`${filePath}: migration files must use LF line endings.`);
    failed = true;
  }
}

if (!baselineTag) {
  console.log("No stable release tag exists yet; there are no released migrations to protect.");
} else {
  if (!stableTagVersion(baselineTag)) {
    console.error(`${baselineTag}: baseline must be a stable SemVer tag such as v1.2.3.`);
    process.exit(2);
  }

  const baselineExists = runGit(["rev-parse", "--verify", "--quiet", `refs/tags/${baselineTag}^{commit}`], {
    allowFailure: true
  });
  if (baselineExists.status !== 0) {
    console.error(`${baselineTag}: baseline tag does not exist locally. Fetch release tags and try again.`);
    process.exit(2);
  }

  const baselineIsAncestor = runGit(["merge-base", "--is-ancestor", `${baselineTag}^{commit}`, "HEAD"], {
    allowFailure: true
  });
  if (baselineIsAncestor.status !== 0) {
    console.error(`${baselineTag}: baseline release is not contained in HEAD. Update the branch and try again.`);
    process.exit(1);
  }

  const releasedMigrationFiles = nullSeparatedGitOutput([
    "ls-tree",
    "-r",
    "--name-only",
    "-z",
    baselineTag,
    "--",
    migrationsPath
  ]).filter((filePath) => filePath.endsWith(".sql"));
  const releasedMigrationFileSet = new Set(releasedMigrationFiles);

  for (const filePath of releasedMigrationFiles) {
    const baselineEntry = nullSeparatedGitOutput(["ls-tree", "-z", baselineTag, "--", filePath])[0];
    const currentEntry = nullSeparatedGitOutput(["ls-tree", "-z", "HEAD", "--", filePath])[0];

    if (!currentEntry) {
      console.error(`${filePath}: released in ${baselineTag} and cannot be deleted or renamed.`);
      failed = true;
    } else if (currentEntry.split("\t", 1)[0] !== baselineEntry.split("\t", 1)[0]) {
      console.error(`${filePath}: released in ${baselineTag} and cannot be modified.`);
      failed = true;
    } else {
      const workingTreeDiff = runGit(["diff", "--quiet", "--no-ext-diff", baselineTag, "--", filePath], {
        allowFailure: true
      });
      if (workingTreeDiff.status === 1) {
        console.error(`${filePath}: released in ${baselineTag} and differs in the working tree or index.`);
        failed = true;
      } else if (workingTreeDiff.status !== 0) {
        process.stderr.write(workingTreeDiff.stderr);
        process.exit(workingTreeDiff.status ?? 1);
      }
    }
  }

  if (!failed) {
    const editableMigrationFiles = currentMigrationFiles.filter((filePath) => !releasedMigrationFileSet.has(filePath));
    console.log(`Released migrations match ${baselineTag}.`);
    if (editableMigrationFiles.length === 0) {
      console.log("No existing migration files can be modified before the next release.");
    } else {
      console.log("Migration files that can still be modified before the next release:");
      for (const filePath of editableMigrationFiles) console.log(`- ${filePath}`);
    }
  }
}

if (failed) {
  console.error("Restore each released migration and add a new numbered migration instead.");
  process.exitCode = 1;
}
