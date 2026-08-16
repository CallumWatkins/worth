import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
// eslint-disable-next-line test/no-import-node-test -- These standalone script tests intentionally use only the Node.js test runner.
import test from "node:test";
import { fileURLToPath } from "node:url";

const checkerPath = fileURLToPath(new URL("./check-migrations.mjs", import.meta.url));
const repositories = [];

async function createRepository() {
  const repository = await mkdtemp(join(tmpdir(), "worth-migration-check-"));
  repositories.push(repository);
  execFileSync("git", ["init", "--quiet"], { cwd: repository });
  execFileSync("git", ["config", "user.email", "migration-test@example.invalid"], { cwd: repository });
  execFileSync("git", ["config", "user.name", "Migration Test"], { cwd: repository });
  execFileSync("git", ["config", "commit.gpgSign", "false"], { cwd: repository });
  execFileSync("git", ["config", "tag.gpgSign", "false"], { cwd: repository });
  await writeFile(join(repository, ".gitattributes"), "* text=auto eol=lf\nsrc-tauri/db/migrations/*.sql text eol=lf\n");
  return repository;
}

async function writeMigration(repository, name, sql) {
  const filePath = join(repository, "src-tauri", "db", "migrations", name);
  await mkdir(dirname(filePath), { recursive: true });
  await writeFile(filePath, sql);
}

function commit(repository, message) {
  execFileSync("git", ["add", "."], { cwd: repository });
  execFileSync("git", ["commit", "--quiet", "-m", message], { cwd: repository });
}

function runChecker(repository, ...args) {
  return spawnSync(process.execPath, [checkerPath, ...args], {
    cwd: repository,
    encoding: "utf8"
  });
}

test.after(async () => {
  await Promise.all(repositories.map((repository) => rm(repository, { recursive: true, force: true })));
});

test("allows migrations to change until they are released", async () => {
  const repository = await createRepository();
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\n");
  commit(repository, "initial migration");

  const initialResult = runChecker(repository);
  assert.equal(initialResult.status, 0);
  assert.match(initialResult.stdout, /No stable release tag exists yet/);

  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY, name TEXT);\n");
  commit(repository, "edit unreleased migration");

  assert.equal(runChecker(repository).status, 0);
});

test("rejects modifications to a released migration", async () => {
  const repository = await createRepository();
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\n");
  commit(repository, "initial migration");
  execFileSync("git", ["tag", "v1.0.0"], { cwd: repository });

  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY, name TEXT);\n");

  const uncommittedResult = runChecker(repository);
  assert.equal(uncommittedResult.status, 1);
  assert.match(uncommittedResult.stderr, /differs in the working tree or index/);

  commit(repository, "mutate released migration");

  const result = runChecker(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /released in v1\.0\.0 and cannot be modified/);
});

test("rejects deletion or renaming of a released migration", async () => {
  const repository = await createRepository();
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\n");
  commit(repository, "initial migration");
  execFileSync("git", ["tag", "v1.0.0"], { cwd: repository });
  execFileSync("git", ["mv", "src-tauri/db/migrations/0001_init.sql", "src-tauri/db/migrations/0002_init.sql"], {
    cwd: repository
  });
  commit(repository, "rename released migration");

  const result = runChecker(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /cannot be deleted or renamed/);
});

test("freezes newly added migrations at the next release tag", async () => {
  const repository = await createRepository();
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\n");
  commit(repository, "initial migration");
  execFileSync("git", ["tag", "v1.0.0"], { cwd: repository });

  await writeMigration(repository, "0002_name.sql", "ALTER TABLE example ADD COLUMN name TEXT;\n");
  commit(repository, "add second migration");
  await writeMigration(repository, "0002_name.sql", "ALTER TABLE example ADD COLUMN display_name TEXT;\n");
  commit(repository, "edit second migration before release");

  const beforeReleaseResult = runChecker(repository);
  assert.equal(beforeReleaseResult.status, 0);
  assert.match(
    beforeReleaseResult.stdout,
    /Migration files that can still be modified before the next release:\r?\n- src-tauri\/db\/migrations\/0002_name\.sql/
  );

  execFileSync("git", ["tag", "v1.1.0"], { cwd: repository });
  await writeMigration(repository, "0002_name.sql", "ALTER TABLE example ADD COLUMN final_name TEXT;\n");
  commit(repository, "edit second migration after release");

  assert.equal(runChecker(repository).status, 1);
  assert.equal(runChecker(repository, "--exclude-tag", "v1.1.0").status, 0);
});

test("rejects non-LF migration files", async () => {
  const repository = await createRepository();
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\r\n");
  execFileSync("git", ["add", "."], { cwd: repository });
  execFileSync("git", ["commit", "--quiet", "-m", "initial migration"], { cwd: repository });
  await writeMigration(repository, "0001_init.sql", "CREATE TABLE example (id INTEGER PRIMARY KEY);\r\n");

  const result = runChecker(repository);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /migration files must use LF line endings/);
});
