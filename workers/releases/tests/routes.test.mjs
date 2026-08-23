import assert from "node:assert/strict";
// eslint-disable-next-line test/no-import-node-test -- These standalone worker tests intentionally use only the Node.js test runner.
import { after, before, test } from "node:test";
import { fileURLToPath } from "node:url";
import { createTestHarness } from "wrangler";

const server = createTestHarness({
  root: fileURLToPath(new URL("../", import.meta.url)),
  workers: [{
    configPath: "wrangler.toml",
    prebuiltWorkerDir: "dist"
  }]
});

before(() => server.listen());
after(() => server.close());

test("serves the health response with public JSON headers", async () => {
  const response = await server.fetch("/health");

  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), { ok: true });
  assert.equal(response.headers.get("access-control-allow-origin"), "*");
  assert.equal(response.headers.get("cache-control"), "no-store");
  assert.equal(response.headers.get("content-type"), "application/json; charset=utf-8");
});

test("redirects versioned downloads to the matching GitHub release asset", async () => {
  const response = await server.fetch(
    "/v1/download/v1.2.3/Worth_1.2.3_x64-setup.exe",
    { redirect: "manual" }
  );

  assert.equal(response.status, 302);
  assert.equal(
    response.headers.get("location"),
    "https://github.com/CallumWatkins/worth/releases/download/v1.2.3/Worth_1.2.3_x64-setup.exe"
  );
  assert.equal(response.headers.get("access-control-allow-origin"), "*");
  assert.equal(response.headers.get("cache-control"), "public, max-age=3600");
});

test("supports HEAD downloads without a response body", async () => {
  const response = await server.fetch(
    "/v1/download/v1.2.3/Worth_1.2.3_x64-setup.exe",
    { method: "HEAD", redirect: "manual" }
  );

  assert.equal(response.status, 302);
  assert.equal(await response.text(), "");
});

test("rejects unsafe download parameters", async () => {
  const invalidVersion = await server.fetch("/v1/download/release!/Worth.exe");
  const invalidFilename = await server.fetch("/v1/download/v1.2.3/Worth..exe");

  assert.equal(invalidVersion.status, 400);
  assert.equal(await invalidVersion.text(), "Invalid release version");
  assert.equal(invalidFilename.status, 400);
  assert.equal(await invalidFilename.text(), "Invalid release filename");
});
