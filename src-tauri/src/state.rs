use sqlx::SqlitePool;

use crate::updates::AppUpdateManager;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub updates: AppUpdateManager,
}
