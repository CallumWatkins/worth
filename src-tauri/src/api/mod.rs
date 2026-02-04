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

