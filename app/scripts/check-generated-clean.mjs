import { spawnSync } from "node:child_process";

const generatedPath = "app/generated";

const status = spawnSync("git", ["status", "--porcelain", "--", generatedPath], {
  encoding: "utf8"
});

if (status.error) throw status.error;
if (status.status !== 0) process.exit(status.status ?? 1);

if (status.stdout.trim().length > 0) {
  spawnSync("git", ["diff", "--", generatedPath], { stdio: "inherit" });
  console.error(`${generatedPath} is not up to date. Run bun run contracts:gen and commit the result.`);
  console.error(status.stdout.trimEnd());
  process.exitCode = 1;
} else {
  console.log(`${generatedPath} is up to date.`);
}
