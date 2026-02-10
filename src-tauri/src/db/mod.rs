pub mod rows;

use chrono::NaiveDate;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    QueryBuilder, Sqlite, SqlitePool,
};
use std::path::PathBuf;
use std::time::Duration;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub async fn init_pool(app: &tauri::AppHandle) -> tauri::Result<SqlitePool> {
    let db_path: PathBuf = app
        .path()
        .resolve("db/worth.sqlite", BaseDirectory::AppLocalData)?;

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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountListRow {
    pub id: i64,
    pub name: String,
    pub currency_code: String,
    pub normal_balance_sign: i32,
    pub opened_date: Option<NaiveDate>,
    pub closed_date: Option<NaiveDate>,

    pub institution_id: i64,
    pub institution_name: String,

    pub type_id: i64,
    pub type_name: String, // e.g. "current"

    pub first_snapshot_date: Option<NaiveDate>,
    pub latest_snapshot_date: Option<NaiveDate>,
    pub latest_balance_minor: Option<i64>,
}

pub async fn accounts_list_full(pool: &SqlitePool) -> Result<Vec<AccountListRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AccountListRow>(
        r"
        SELECT
            a.id,
            a.name,
            a.currency_code,
            a.normal_balance_sign,
            a.opened_date,
            a.closed_date,
            i.id AS institution_id,
            i.name AS institution_name,
            t.id AS type_id,
            t.name AS type_name,
            first.balance_date AS first_snapshot_date,
            latest.balance_date AS latest_snapshot_date,
            latest.balance_minor AS latest_balance_minor
        FROM
            accounts AS a
            INNER JOIN institutions AS i ON i.id = a.institution_id
            INNER JOIN account_types AS t ON t.id = a.type_id
            LEFT JOIN (
                SELECT
                    account_id,
                    MIN(balance_date) AS balance_date
                FROM
                    account_balance_snapshots
                GROUP BY
                    account_id
            ) AS FIRST ON first.account_id = a.id
            LEFT JOIN (
                SELECT
                    abs.account_id,
                    abs.balance_date,
                    abs.balance_minor
                FROM
                    account_balance_snapshots AS abs
                    INNER JOIN (
                        SELECT
                            account_id,
                            MAX(balance_date) AS max_date
                        FROM
                            account_balance_snapshots
                        GROUP BY
                            account_id
                    ) AS m ON m.account_id = abs.account_id
                    AND m.max_date = abs.balance_date
            ) AS latest ON latest.account_id = a.id
        ORDER BY
            a.name ASC
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn account_get_full(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Option<AccountListRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, AccountListRow>(
        r"
        SELECT
            a.id,
            a.name,
            a.currency_code,
            a.normal_balance_sign,
            a.opened_date,
            a.closed_date,
            i.id AS institution_id,
            i.name AS institution_name,
            t.id AS type_id,
            t.name AS type_name,
            first.balance_date AS first_snapshot_date,
            latest.balance_date AS latest_snapshot_date,
            latest.balance_minor AS latest_balance_minor
        FROM
            accounts AS a
            INNER JOIN institutions AS i ON i.id = a.institution_id
            INNER JOIN account_types AS t ON t.id = a.type_id
            LEFT JOIN (
                SELECT
                    account_id,
                    MIN(balance_date) AS balance_date
                FROM
                    account_balance_snapshots
                GROUP BY
                    account_id
            ) AS FIRST ON first.account_id = a.id
            LEFT JOIN (
                SELECT
                    abs.account_id,
                    abs.balance_date,
                    abs.balance_minor
                FROM
                    account_balance_snapshots AS abs
                    INNER JOIN (
                        SELECT
                            account_id,
                            MAX(balance_date) AS max_date
                        FROM
                            account_balance_snapshots
                        GROUP BY
                            account_id
                    ) AS m ON m.account_id = abs.account_id
                    AND m.max_date = abs.balance_date
            ) AS latest ON latest.account_id = a.id
        WHERE
            a.id = ?
        ",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountSnapshotRow {
    pub account_id: i64,
    pub balance_date: NaiveDate,
    pub balance_minor: i64,
}

pub async fn snapshots_for_account(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Vec<rows::AccountBalanceSnapshotRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, rows::AccountBalanceSnapshotRow>(
        r"
        SELECT
            id,
            account_id,
            balance_date,
            balance_minor,
            created_at
        FROM
            account_balance_snapshots
        WHERE
            account_id = ?
        ORDER BY
            balance_date DESC
        ",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn snapshots_for_accounts_between(
    pool: &SqlitePool,
    account_ids: &[i64],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<AccountSnapshotRow>, sqlx::Error> {
    if account_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::<Sqlite>::new(
        "SELECT account_id, balance_date, balance_minor \
         FROM account_balance_snapshots \
         WHERE balance_date >= ",
    );
    qb.push_bind(start_date);
    qb.push(" AND balance_date <= ");
    qb.push_bind(end_date);
    qb.push(" AND account_id IN (");

    {
        let mut separated = qb.separated(", ");
        for id in account_ids {
            separated.push_bind(id);
        }
    }
    qb.push(")");

    qb.push(" ORDER BY account_id ASC, balance_date ASC");

    let rows = qb
        .build_query_as::<AccountSnapshotRow>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn last_snapshots_before(
    pool: &SqlitePool,
    account_ids: &[i64],
    start_date: NaiveDate,
) -> Result<Vec<AccountSnapshotRow>, sqlx::Error> {
    if account_ids.is_empty() {
        return Ok(Vec::new());
    }

    // For each account: the latest snapshot strictly before `start_date`.
    let mut qb = QueryBuilder::<Sqlite>::new(
        r"
        SELECT
            s.account_id,
            s.balance_date,
            s.balance_minor
        FROM
            account_balance_snapshots AS s
            INNER JOIN (
                SELECT
                    account_id,
                    MAX(balance_date) AS max_date
                FROM
                    account_balance_snapshots
                WHERE
                    balance_date < ",
    );
    qb.push_bind(start_date);
    qb.push(" AND account_id IN (");
    {
        let mut separated = qb.separated(", ");
        for id in account_ids {
            separated.push_bind(id);
        }
    }
    qb.push(")");
    qb.push(
        r"
            GROUP BY account_id
        ) AS m
        ON m.account_id = s.account_id
        AND m.max_date = s.balance_date
        ORDER BY s.account_id ASC
        ",
    );

    let rows = qb
        .build_query_as::<AccountSnapshotRow>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn earliest_snapshot_date(pool: &SqlitePool) -> Result<Option<NaiveDate>, sqlx::Error> {
    let min_date: Option<NaiveDate> =
        sqlx::query_scalar("SELECT MIN(balance_date) FROM account_balance_snapshots")
            .fetch_one(pool)
            .await?;
    Ok(min_date)
}

pub async fn earliest_snapshot_date_for_account(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Option<NaiveDate>, sqlx::Error> {
    let min_date: Option<NaiveDate> = sqlx::query_scalar(
        r"
        SELECT
            MIN(balance_date)
        FROM
            account_balance_snapshots
        WHERE
            account_id = ?
        ",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await?;

    Ok(min_date)
}
