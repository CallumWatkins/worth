use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use tauri::State;

use crate::state::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ApiError {
    #[error("Database error")]
    Db,
    #[error("Not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
}

#[tauri::command]
#[specta::specta]
pub async fn hello(name: String) -> Result<String, ApiError> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    if name.is_empty() {
        return Err(ApiError::Validation("Name cannot be empty".into()));
    }
    Ok(format!("Hello, {name}!"))
}

pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    use tauri_specta::{collect_commands, Builder};

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        hello
    ]);

    #[cfg(debug_assertions)]
    {
        use std::path::PathBuf;

        let bindings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("app")
            .join("bindings.ts");

        builder
            .export(
                Typescript::default().bigint(BigIntExportBehavior::Number),
                bindings_path,
            )
            .expect("Failed to export typescript bindings");
    }

    builder.invoke_handler()
}
