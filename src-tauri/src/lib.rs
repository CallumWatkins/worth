#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

extern crate self as worth_lib;

pub mod api;
pub mod contracts;
mod db;
mod state;

pub use worth_macros::export_schema;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
