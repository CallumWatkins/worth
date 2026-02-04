pub mod rows;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::path::PathBuf;
use std::time::Duration;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub async fn init_pool(app: &tauri::AppHandle) -> tauri::Result<SqlitePool> {
    let db_path: PathBuf = app
        .path()
        .resolve("worth.sqlite", BaseDirectory::AppLocalData)?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;

    Ok(pool)
}

