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
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));
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
        .invoke_handler(api::invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
