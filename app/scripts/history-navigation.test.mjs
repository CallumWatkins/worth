import assert from "node:assert/strict";
// eslint-disable-next-line test/no-import-node-test -- Use the repository's standalone Node.js test runner.
import { test } from "node:test";
import { ref, shallowRef } from "vue";
import { createHistoryNavigation } from "../utils/history-navigation.ts";

function setup(paths = ["/", "/accounts"], initialIndex = paths.length - 1) {
  let index = initialIndex;
  const before = new Set();
  const after = new Set();
  const errors = new Set();
  const requested = [];
  const route = ref({ fullPath: paths[index] });
  const layer = shallowRef(null);
  const confirmation = ref(false);
  const history = {
    get location() {
      return paths[index];
    },
    get state() {
      return { back: paths[index - 1] ?? null, forward: paths[index + 1] ?? null };
    }
  };
  let closeCount = 0;
  const nav = createHistoryNavigation({
    options: { history },
    currentRoute: route,
    back: () => requested.push(-1),
    forward: () => requested.push(1),
    beforeEach: (guard) => {
      before.add(guard);
      return () => before.delete(guard);
    },
    afterEach: (guard) => {
      after.add(guard);
      return () => after.delete(guard);
    },
    onError: (guard) => {
      errors.add(guard);
      return () => errors.delete(guard);
    }
  }, {
    getTopLayer: () => layer.value,
    closeTopLayerForBack: async () => {
      closeCount++;
      layer.value = null;
      return "handled";
    }
  }, confirmation, () => {
    if (!confirmation.value) return false;
    confirmation.value = false;
    return true;
  });

  return {
    nav,
    layer,
    confirmation,
    requested,
    before,
    after,
    errors,
    get closeCount() {
      return closeCount;
    },
    pop: (delta) => {
      index += delta;
      nav.onPopState();
    },
    commit: () => {
      route.value = { fullPath: history.location };
      after.forEach((fn) => fn());
    },
    reject: () => after.forEach((fn) => fn()),
    push: (path) => {
      paths.splice(index + 1, paths.length, path);
      index++;
      route.value = { fullPath: path };
      after.forEach((fn) => fn());
    },
    replace: (path) => {
      paths[index] = path;
      route.value = { fullPath: path };
      after.forEach((fn) => fn());
    }
  };
}

test("initial entry disables both directions", async () => {
  const { nav, requested } = setup(["/"]);
  assert.equal(nav.canGoBack.value, false);
  assert.equal(nav.canGoForward.value, false);
  await nav.back();
  nav.forward();
  assert.deepEqual(requested, []);
});

test("Back/Forward update only after navigation settles and reject repeated presses", async () => {
  const h = setup();
  await h.nav.back();
  await h.nav.back();
  assert.deepEqual(h.requested, [-1]);
  assert.equal(h.nav.canGoBack.value, false);
  assert.equal(h.nav.backAvailable.value, true);
  h.pop(-1);
  assert.equal(h.nav.canGoForward.value, false);
  assert.equal(h.nav.backAvailable.value, true);
  h.commit();
  assert.equal(h.nav.canGoBack.value, false);
  assert.equal(h.nav.backAvailable.value, false);
  assert.equal(h.nav.canGoForward.value, true);
  h.nav.forward();
  h.pop(1);
  h.commit();
  assert.equal(h.nav.canGoBack.value, true);
  assert.equal(h.nav.canGoForward.value, false);
});

test("route changes keep arrow availability stable while blocking history input", async () => {
  const h = setup(["/", "/accounts", "/institutions"], 1);
  h.before.forEach((guard) => guard());
  assert.equal(h.nav.backAvailable.value, true);
  assert.equal(h.nav.forwardAvailable.value, true);
  assert.equal(h.nav.canGoBack.value, false);
  assert.equal(h.nav.canGoForward.value, false);
  await h.nav.back();
  h.nav.forward();
  assert.deepEqual(h.requested, []);
  h.push("/settings");
  assert.equal(h.nav.backAvailable.value, true);
  assert.equal(h.nav.forwardAvailable.value, false);
  assert.equal(h.nav.canGoBack.value, true);
});

test("new navigation truncates Forward, while replace preserves it", () => {
  const h = setup(["/", "/accounts", "/institutions"], 1);
  h.replace("/accounts?label=2");
  assert.equal(h.nav.canGoBack.value, true);
  assert.equal(h.nav.canGoForward.value, true);
  h.push("/settings");
  assert.equal(h.nav.canGoForward.value, false);
});

test("cancelled history navigation stays disabled until the router rollback", async () => {
  const h = setup();
  await h.nav.back();
  h.pop(-1);
  h.reject();
  assert.equal(h.nav.canGoForward.value, false);
  assert.equal(h.nav.canGoBack.value, false);
  h.pop(1);
  assert.equal(h.nav.canGoForward.value, false);
  assert.equal(h.nav.canGoBack.value, true);
});

test("Back closes an open layer even at the first history entry", async () => {
  const h = setup(["/"]);
  h.layer.value = { open: true, close() {} };
  assert.equal(h.nav.canGoBack.value, true);
  await h.nav.back();
  assert.equal(h.closeCount, 1);
  assert.deepEqual(h.requested, []);
  assert.equal(h.nav.canGoBack.value, false);
});

test("pending layers block Back and all open layers block Forward", async () => {
  const h = setup(["/", "/accounts", "/institutions"], 1);
  const pending = ref(true);
  h.layer.value = { open: true, pending, close() {} };
  await h.nav.back();
  h.nav.forward();
  assert.equal(h.nav.canGoBack.value, false);
  assert.equal(h.nav.canGoForward.value, false);
  assert.equal(h.closeCount, 0);
  assert.deepEqual(h.requested, []);
  pending.value = false;
  assert.equal(h.nav.canGoBack.value, true);
  assert.equal(h.nav.canGoForward.value, false);
});

test("Back cancels the active discard confirmation without navigating", async () => {
  const h = setup();
  await h.nav.back();
  h.pop(-1);
  h.confirmation.value = true;
  assert.equal(h.nav.canGoBack.value, true);
  assert.equal(h.nav.canGoForward.value, false);
  await h.nav.back();
  assert.equal(h.confirmation.value, false);
  assert.deepEqual(h.requested, [-1]);
});

test("disposal removes every registered router hook", () => {
  const h = setup();
  h.nav.dispose();
  assert.equal(h.before.size + h.after.size + h.errors.size, 0);
});
