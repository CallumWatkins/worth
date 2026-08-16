# Automatic updates

Rust interacts with the Tauri Updater plugin to check for, download, and install updates. It manages this state and exposes it to the frontend through an event channel.

## Key files

| Area | File |
| --- | --- |
| Rust update flow and DTOs | `src-tauri/src/updates.rs` |
| Tauri updater config | `src-tauri/tauri.conf.json5` and `src-tauri/tauri.dev.conf.json` |
| Frontend update state and commands | `app/composables/useAppUpdates.ts` |
| App-level update event listener | `app/plugins/app-updates.client.ts` |
| Settings row view model | `app/composables/useAppUpdateRow.ts` |

## Ownership

Rust owns checking, downloading, pending downloaded update data, installing, and the update state emitted to the frontend.

The frontend owns the Settings row presentation. It fetches/listens for Rust state and maps it to friendly copy and controls.

## Build behavior

| Build | Updater plugin | `supports_updates` | Startup check | Settings row |
| --- | --- | --- | --- | --- |
| Debug/dev | Not registered | `false` | Never runs | `Updates are disabled for this installation.` |
| Production with supported updater target | Registered | `true` | Runs on app startup | Real update state |
| Production without supported updater target | Registered | `false` | Never runs | `Updates are disabled for this installation.` |

Debug builds set the updater config to `null` and do not register the updater plugin.

## IPC commands

| Command | Purpose |
| --- | --- |
| `app_updates_state_get` | Returns the current Rust update state. |
| `app_updates_check` | Runs a user-initiated check. An available update is downloaded and left pending. |
| `app_updates_install_pending_and_restart` | Installs the pending update and restarts/relaunches according to platform behavior. |

## Platform behavior

| Platform | Startup update | Manual update |
| --- | --- | --- |
| Windows | Download and wait for `Update and restart Worth`. | Download and wait for `Update and restart Worth`. |
| macOS/Linux | Download, install, then wait for `Restart Worth`. | Download and wait for `Update and restart Worth`. |

On Windows, the installer handles closing and relaunching after install. On macOS/Linux startup checks, Rust installs automatically and the user finishes by restarting. Manual checks never install immediately.

Only one update operation can run at a time. Starting a check clears any pending download; an install failure preserves it for retry. Pending update bytes are held in memory. Installing with nothing pending emits `no_pending_update`; success emits `installed`.

`useAppUpdatesManager` is the low-level frontend API. The app plugin listens once per session for `worth://updates/state` and updates shared state and the TanStack Query cache. `useAppUpdateRow` projects that state into Settings UI.
