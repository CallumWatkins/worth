#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

extern crate self as worth_lib;

pub mod api;
pub mod contracts;
mod db;
mod imports;
mod state;

pub use worth_macros::export_schema;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = api::specta_builder();
    let mut builder = tauri::Builder::default()
        .plugin(prevent_default())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    }

    builder
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init_pool(&handle).await?;
                sqlx::migrate!("./db/migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
                handle.manage(AppState { pool });
                Ok::<(), tauri::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(debug_assertions)]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    let mut builder = tauri_plugin_prevent_default::Builder::new();
    #[cfg(target_os = "windows")]
    {
        use tauri_plugin_prevent_default::PlatformOptions;

        builder = builder.platform(
            PlatformOptions::new()
                .default_script_dialogs(false)
                .general_autofill(false)
                .host_objects(false)
                .password_autosave(false)
                .pinch_zoom(false)
                .swipe_navigation(false)
                .zoom_control(false),
        )
    }
    builder
        .with_flags(
            Flags::all().difference(
                Flags::CONTEXT_MENU | Flags::DEV_TOOLS | Flags::RELOAD | Flags::FOCUS_MOVE,
            ),
        )
        .build()
}

#[cfg(not(debug_assertions))]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    let mut builder = tauri_plugin_prevent_default::Builder::new();
    #[cfg(target_os = "windows")]
    {
        use tauri_plugin_prevent_default::PlatformOptions;

        builder = builder.platform(
            PlatformOptions::new()
                .default_context_menus(false)
                .default_script_dialogs(false)
                .dev_tools(false)
                .general_autofill(false)
                .host_objects(false)
                .password_autosave(false)
                .pinch_zoom(false)
                .swipe_navigation(false)
                .zoom_control(false),
        )
    }
    builder
        .with_flags(Flags::all().difference(Flags::RELOAD | Flags::FOCUS_MOVE))
        .build()
}
