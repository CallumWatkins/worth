# History navigation safety

Worth supports browser-style Back and Forward inputs in its Tauri WebView, including `Alt+Left`, `Alt+Right`, and mouse side buttons. Navigation handling must protect in-progress UI without turning modal state into route history.

## Expected behavior

- Back handles the topmost open modal or slideover before navigating the route.
- A clean layer closes immediately.
- A dirty layer asks for confirmation before closing.
- A pending layer blocks navigation silently.
- Forward does nothing while a layer is open.
- With no open layer, normal router history applies.
- Explicit navigation closes all clean layers, confirms dirty layers, and stops at any pending layer.

Opening a modal must not push a history entry, change the URL, or clear Forward history.

## Choosing a primitive

| Situation | Primitive |
| --- | --- |
| Controlled modal or slideover | `useNavigationLayer` |
| Dirty page-level form | `usePreventRouteNavigation` |
| Pending page-level operation | `useBlockRouteNavigationWhile` |
| Dropdown, popover, select menu, date picker, or search suggestions | None |

Small transient overlays are outside overlay-first Back behavior. Clear their state after successful route changes when they could otherwise remain visible.

## Registering a layer

Register a modal or slideover in the component that owns its controlled open state:

```ts
useNavigationLayer({
  id: "account-create-dialog",
  open,
  dirty: computed(() => form.value?.dirty ?? false),
  pending: computed(() => createAccount.isPending),
  close: () => {
    open.value = false;
  },
  discardTitle: "Discard account changes?"
});
```

The registration is removed with the component scope. Topmost order is based on when a layer opens, not component mount order.

Keep dirty and pending calculations owned by the workflow:

- Dirty means the user changed or entered data. Prefilled defaults are not dirty.
- Delete confirmation text does not make a delete dialog dirty.
- Pending includes any save, create, update, delete, or import that must finish before the user leaves.
- Close the layer before navigating after a successful operation.

Use controlled component state. Do not inspect or manipulate Nuxt UI overlay DOM.

## Route and resource considerations

Pages reused across route parameters must reset resource-specific layer state when the resource ID changes. The navigation guards handle dirty or pending layers before that reset occurs.

Deleting the resource represented by the current route requires additional care:

1. Delete without invalidating the active resource query.
2. Close the dialog and disable obsolete page dirty-state protection.
3. Navigate away with `replace: true`.
4. Invalidate affected queries after navigation.

This prevents both a dead settings history entry and a refetch-driven 404 before redirect. Shared mutation composables should remain route-agnostic; the caller chooses whether invalidation is immediate or delayed.

Fatal resource and catch-all errors are allowed for genuinely stale history entries. Navigation to a valid route must clear the Nuxt error state; this is handled centrally by `app/plugins/history-navigation.client.ts`.

## Extending or reviewing a workflow

When adding or changing a modal, slideover, guarded page, or redirect:

- Select the appropriate primitive from the table above.
- Verify clean, dirty, and pending behavior separately.
- Verify both native Back and intentional navigation such as links or shortcuts.
- Verify Forward cannot navigate behind an open layer.
- If the page is reused for another resource ID, verify no layer state survives the change.
- If deleting the current resource, verify Back does not return to its deleted settings route.
- Run `bun run check:ts` and test the behavior in the Tauri WebView.

The registry and input coordinator are intentionally domain-agnostic. Do not add account, institution, snapshot, settings, analytics, or query-key logic to `useNavigationLayers` or `history-navigation.client.ts`.
