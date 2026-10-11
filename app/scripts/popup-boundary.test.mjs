import assert from "node:assert/strict";
// eslint-disable-next-line test/no-import-node-test -- Use the repository's standalone Node.js test runner.
import { test } from "node:test";
import { computePosition, flip, offset, shift, size } from "@floating-ui/core";
import { createApp, shallowRef } from "vue";
import { appViewportKey, useAppPopupContent } from "../composables/useAppPopupContent.ts";

test("an upward menu fits below the title bar in a minimum-height window", async () => {
  const windowBounds = { x: 0, y: 0, width: 850, height: 300 };
  const viewport = shallowRef({ x: 0, y: 32, width: 850, height: 268 });
  const app = createApp({});
  app.provide(appViewportKey, viewport);
  const content = app.runWithContext(() => useAppPopupContent({ align: "end" }));

  const positionMenu = async (boundary) => {
    let height = 220;
    const collisions = { boundary, padding: 8, altBoundary: Boolean(boundary) };
    const result = await computePosition({}, {}, {
      placement: `bottom-${content.value.align}`,
      strategy: "fixed",
      platform: {
        getElementRects: () => ({
          reference: { x: 100, y: 250, width: 150, height: 32 },
          floating: { x: 0, y: 0, width: 150, height }
        }),
        getClippingRect: ({ boundary }) => typeof boundary === "object" ? boundary : windowBounds,
        getDimensions: () => ({ width: 150, height })
      },
      // Reka's positioning order, including Nuxt UI's available-height limit.
      middleware: [
        offset(8),
        shift(collisions),
        flip(collisions),
        size({ ...collisions, apply: ({ availableHeight }) => {
          height = Math.min(220, availableHeight);
        } })
      ]
    });
    return { ...result, height };
  };

  const before = await positionMenu(undefined);
  assert.equal(before.y, 22, "whole-window collisions reproduce the clipped menu");
  const after = await positionMenu(content.value.collisionBoundary);
  assert.equal(after.placement, "top-end");
  assert.equal(after.y, 40);
  assert.equal(after.height, 202);
  assert.ok(after.y + after.height <= 300);

  // Shell replacement after a fatal error must update existing reactive content props.
  viewport.value = { x: 0, y: 40, width: 850, height: 260 };
  const remounted = await positionMenu(content.value.collisionBoundary);
  assert.equal(remounted.y, 48);
});
