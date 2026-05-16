# Automatic Updates

Rust interacts with the Tauri Updater plugin to check for, download, and install updates. It manages this state and exposes it to the frontend through an event channel.

## Key Files

| Area | File |
| --- | --- |
| Rust update flow and DTOs | `src-tauri/src/updates.rs` |
| Tauri updater config | `src-tauri/tauri.conf.json` and `src-tauri/tauri.dev.conf.json` |
| Frontend update state and commands | `app/composables/useAppUpdates.ts` |
| App-level update event listener | `app/plugins/app-updates.client.ts` |
| Settings row view model | `app/composables/useAppUpdateRow.ts` |

## Ownership

Rust owns checking, downloading, pending downloaded update data, installing, and the update state emitted to the frontend.

The frontend owns the Settings row presentation. It fetches/listens for Rust state and maps it to friendly copy and controls.

## Build Behavior

| Build | Updater Plugin | `supports_updates` | Startup Check | Settings Row |
| --- | --- | --- | --- | --- |
| Debug/dev | Not registered | `false` | Never runs | `Updates are disabled for this installation.` |
| Production with supported updater target | Registered | `true` | Runs on app startup | Real update state |
| Production without supported updater target | Registered | `false` | Never runs | `Updates are disabled for this installation.` |

Debug builds set the updater config to `null` and do not register the updater plugin.

## IPC Commands

| Command | Purpose |
| --- | --- |
| `app_updates_state_get` | Returns the current Rust update state. |
| `app_updates_check` | Runs a user-initiated update check. If an update is available, downloads it and leaves it pending. |
| `app_updates_install_pending_and_restart` | Installs the pending downloaded update and restarts/relaunches according to platform behavior. |

## Startup Checks

Production builds check for updates on startup when updates are supported. A startup check may finish as `up_to_date`, emit `downloading` progress, or emit an `error` state. If an update downloads successfully, behavior differs by platform.

## Platform Behavior

| Platform | Startup Update | Manual Update |
| --- | --- | --- |
| Windows | Download and wait for `Update and restart Worth`. | Download and wait for `Update and restart Worth`. |
| macOS/Linux | Download, install, then wait for `Restart Worth`. | Download and wait for `Update and restart Worth`. |

On Windows, the installer handles closing/relaunch behavior after install, so Rust does not request an app restart.

On macOS/Linux startup checks, Rust installs automatically and the user finishes the update by clicking `Restart Worth` or restarting the app manually.

## Manual Checks

Manual checks are started from the Settings About row. They check and download, but they never install immediately after download. If an update downloads successfully, it is stored as pending on every platform and the row shows `Update and restart Worth`.

## Install And Restart

`Update and restart Worth` installs the pending downloaded update. If no update is pending, Rust emits `no_pending_update`. If install fails, Rust keeps the pending update available so the user can retry.

Successful installs emit `installed`. On macOS/Linux user-triggered installs, Rust requests a restart. On Windows, the installer handles the platform flow.

## Frontend Data Flow

`useAppUpdatesManager` is the low-level frontend API for state, check, and install commands.

`app/plugins/app-updates.client.ts` listens once per app session for `worth://updates/state` and writes event payloads into the shared state and TanStack Query cache.

`useAppUpdateRow` projects backend state into the Settings row model for `settings.vue` to render.

## Concurrency And Pending Data

Only one update operation can run at a time. Starting a new check clears any pending downloaded update, while install failures keep the pending update available for retry. Pending downloaded update bytes are held in memory.
