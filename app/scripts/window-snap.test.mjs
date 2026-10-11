import assert from "node:assert/strict";
// eslint-disable-next-line test/no-import-node-test -- Use the repository's standalone Node.js test runner.
import { test } from "node:test";
import { trackWindowSnapButton } from "../utils/window-snap.ts";

function setup(t) {
  const frames = new Map();
  let nextFrame = 0;
  const original = Object.fromEntries(["window", "requestAnimationFrame", "cancelAnimationFrame"].map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  Object.defineProperties(globalThis, {
    window: { configurable: true, value: { devicePixelRatio: 1 } },
    requestAnimationFrame: { configurable: true, value: (callback) => {
      frames.set(++nextFrame, callback);
      return nextFrame;
    } },
    cancelAnimationFrame: { configurable: true, value: (id) => frames.delete(id) }
  });
  t.after(() => {
    Object.entries(original).forEach(([key, descriptor]) => {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    });
  });
  const rect = { left: 1200, top: 0, right: 1246, bottom: 32 };
  const button = {
    isConnected: true,
    disabled: false,
    visible: true,
    checkVisibility() {
      return this.visible;
    },
    getBoundingClientRect: () => rect
  };
  const updates = [];
  const stop = trackWindowSnapButton(button, (bounds) => updates.push(bounds));
  return {
    button,
    rect,
    updates,
    stop,
    frames,
    tick() {
      const pending = [...frames.values()];
      frames.clear();
      pending.forEach((callback) => callback());
    }
  };
}

test("unchanged frames do not send IPC; position-only changes do", (t) => {
  const s = setup(t);
  s.tick();
  assert.equal(s.updates.length, 1);
  s.rect.left += 100;
  s.rect.right += 100;
  s.tick();
  assert.deepEqual(s.updates.at(-1), { x: 1300, y: 0, width: 46, height: 32 });
});

test("DPI and WebView zoom changes update physical pixel bounds", (t) => {
  const s = setup(t);
  window.devicePixelRatio = 1.5;
  s.tick();
  assert.deepEqual(s.updates.at(-1), { x: 1800, y: 0, width: 69, height: 48 });
});

test("hidden, disabled and disconnected buttons remove the target", (t) => {
  const s = setup(t);
  ["visible", "disabled", "isConnected"].forEach((property) => {
    const initial = s.button[property];
    s.button[property] = !initial;
    s.tick();
    assert.equal(s.updates.at(-1), null);
    s.button[property] = initial;
    s.tick();
    assert.notEqual(s.updates.at(-1), null);
  });
});

test("zero-sized buttons never create a one-pixel target", (t) => {
  const s = setup(t);
  s.rect.right = s.rect.left;
  s.tick();
  assert.equal(s.updates.at(-1), null);
});

test("unmount removes the target and prevents queued frames from recreating it", (t) => {
  const s = setup(t);
  const queued = [...s.frames.values()][0];
  s.stop();
  assert.equal(s.frames.size, 0);
  const count = s.updates.length;
  queued();
  assert.equal(s.updates.length, count);
  assert.equal(s.updates.at(-1), null);
});
