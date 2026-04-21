pub mod rows;

use chrono::NaiveDate;
use itertools::Itertools;
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InstitutionSummaryRow {
    pub id: i64,
    pub name: String,
    pub account_count: i64,
    pub empty_account_count: i64,
    pub total_balance_minor: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InstitutionAccountTypeRow {
    pub institution_id: i64,
    pub type_name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountDeletePreviewRow {
    pub id: i64,
    pub name: String,
    pub institution_name: String,
    pub snapshot_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InstitutionAccountDeletePreviewRow {
    pub id: i64,
    pub name: String,
    pub snapshot_count: i64,
}

#[derive(Debug, Clone)]
pub enum GlobalSearchRow {
    Account {
        id: i64,
        name: String,
        type_name: String,
        institution_name: String,
    },
    Institution {
        id: i64,
        name: String,
    },
}

fn normalize_search_query(query: &str) -> Option<(String, String)> {
    let mut normalized = String::with_capacity(query.len());
    for ch in query.trim().chars() {
        if ch.is_alphanumeric() {
            normalized.push(ch);
        } else {
            normalized.push(' ');
        }
    }

    let terms: Vec<&str> = normalized.split_whitespace().collect();
    if terms.is_empty() {
        return None;
    }

    let phrase = terms.join(" ");
    let fts_query = terms.into_iter().map(|term| format!("{term}*")).join(" ");
    Some((phrase, fts_query))
}

pub async fn search_global(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<GlobalSearchRow>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct GlobalSearchRawRow {
        kind: String,
        id: i64,
        name: String,
        type_name: Option<String>,
        institution_name: Option<String>,
    }

    let Some((phrase_query, fts_query)) = normalize_search_query(query) else {
        return Ok(Vec::new());
    };

    let rows = sqlx::query_as::<_, GlobalSearchRawRow>(
        r"
        WITH
            matched AS (
                SELECT
                    kind,
                    entity_id,
                    rank AS bm25_rank,
                    CASE
                        WHEN INSTR(LOWER(name), LOWER(?)) > 0 THEN 1
                        ELSE 0
                    END AS has_phrase,
                    CASE
                        WHEN INSTR(LOWER(name), LOWER(?)) > 0 THEN INSTR(LOWER(name), LOWER(?))
                        ELSE 2147483647
                    END AS phrase_pos
                FROM
                    search_fts
                WHERE
                    search_fts MATCH ?
                    AND rank MATCH 'bm25(0.0, 0.0, 10.0, 2.0, 1.0)'
            ),
            account_hits AS (
                SELECT
                    m.bm25_rank,
                    m.has_phrase,
                    m.phrase_pos,
                    'account' AS kind,
                    a.id,
                    a.name,
                    t.name AS type_name,
                    i.name AS institution_name
                FROM
                    matched AS m
                    INNER JOIN accounts AS a ON m.kind = 'account'
                    AND a.id = m.entity_id
                    INNER JOIN account_types AS t ON t.id = a.type_id
                    INNER JOIN institutions AS i ON i.id = a.institution_id
            ),
            institution_hits AS (
                SELECT
                    m.bm25_rank,
                    m.has_phrase,
                    m.phrase_pos,
                    'institution' AS kind,
                    i.id,
                    i.name,
                    NULL AS type_name,
                    NULL AS institution_name
                FROM
                    matched AS m
                    INNER JOIN institutions AS i ON m.kind = 'institution'
                    AND i.id = m.entity_id
            )
        SELECT
            results.kind,
            results.id,
            results.name,
            results.type_name,
            results.institution_name
        FROM
            (
                SELECT
                    *
                FROM
                    account_hits
                UNION ALL
                SELECT
                    *
                FROM
                    institution_hits
            ) AS results
        ORDER BY
            results.has_phrase DESC,
            results.phrase_pos ASC,
            results.bm25_rank ASC,
            CASE
                WHEN results.kind = 'institution' THEN 0
                ELSE 1
            END ASC,
            results.name COLLATE nocase ASC
        LIMIT
            10
        ",
    )
    .bind(&phrase_query)
    .bind(&phrase_query)
    .bind(&phrase_query)
    .bind(fts_query)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| match row.kind.as_str() {
            "account" => {
                let Some(type_name) = row.type_name else {
                    return Err(sqlx::Error::Decode(
                        "search_global: NULL type_name for account row".into(),
                    ));
                };
                let Some(institution_name) = row.institution_name else {
                    return Err(sqlx::Error::Decode(
                        "search_global: NULL institution_name for account row".into(),
                    ));
                };

                Ok(GlobalSearchRow::Account {
                    id: row.id,
                    name: row.name,
                    type_name,
                    institution_name,
                })
            }
            "institution" => Ok(GlobalSearchRow::Institution {
                id: row.id,
                name: row.name,
            }),
            other => Err(sqlx::Error::Decode(
                format!("search_global: unknown kind={other:?}").into(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()
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

pub async fn accounts_list_full_for_institution(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<Vec<AccountListRow>, sqlx::Error> {
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
        WHERE
            a.institution_id = ?
        ORDER BY
            a.name ASC
        ",
    )
    .bind(institution_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn institutions_list_summary(
    pool: &SqlitePool,
) -> Result<Vec<InstitutionSummaryRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, InstitutionSummaryRow>(
        r"
        SELECT
            i.id,
            i.name,
            COUNT(a.id) AS account_count,
            SUM(
                CASE
                    WHEN a.id IS NOT NULL
                    AND COALESCE(latest.balance_minor, 0) = 0 THEN 1
                    ELSE 0
                END
            ) AS empty_account_count,
            COALESCE(SUM(COALESCE(latest.balance_minor, 0)), 0) AS total_balance_minor
        FROM
            institutions AS i
            LEFT JOIN accounts AS a ON a.institution_id = i.id
            LEFT JOIN (
                SELECT
                    abs.account_id,
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
        GROUP BY
            i.id,
            i.name
        ORDER BY
            i.name ASC
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn institutions_account_types(
    pool: &SqlitePool,
) -> Result<Vec<InstitutionAccountTypeRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, InstitutionAccountTypeRow>(
        r"
        SELECT
            a.institution_id,
            t.name AS type_name
        FROM
            accounts AS a
            INNER JOIN account_types AS t ON t.id = a.type_id
        GROUP BY
            a.institution_id,
            t.name
        ORDER BY
            a.institution_id ASC,
            COUNT(a.id) DESC,
            t.name ASC
        ",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn institution_get(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<Option<rows::InstitutionRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, rows::InstitutionRow>(
        r"
        SELECT
            id,
            name
        FROM
            institutions
        WHERE
            id = ?
        ",
    )
    .bind(institution_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
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

pub async fn snapshots_for_account_dates(
    pool: &SqlitePool,
    account_id: i64,
    dates: &[NaiveDate],
) -> Result<Vec<rows::AccountBalanceSnapshotRow>, sqlx::Error> {
    if dates.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = QueryBuilder::<Sqlite>::new(
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
            account_id = ",
    );
    qb.push_bind(account_id);
    qb.push(" AND balance_date IN (");
    {
        let mut separated = qb.separated(", ");
        for date in dates {
            separated.push_bind(date);
        }
    }
    qb.push(") ORDER BY balance_date ASC");

    qb.build_query_as::<rows::AccountBalanceSnapshotRow>()
        .fetch_all(pool)
        .await
}

pub async fn account_snapshot_get(
    pool: &SqlitePool,
    account_id: i64,
    snapshot_id: i64,
) -> Result<Option<rows::AccountBalanceSnapshotRow>, sqlx::Error> {
    sqlx::query_as::<_, rows::AccountBalanceSnapshotRow>(
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
            AND id = ?
        ",
    )
    .bind(account_id)
    .bind(snapshot_id)
    .fetch_optional(pool)
    .await
}

pub async fn account_snapshot_create_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    balance_date: NaiveDate,
    balance_minor: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r"
        INSERT INTO
            account_balance_snapshots (
                account_id,
                balance_date,
                balance_minor
            )
        VALUES
            (?, ?, ?)
        ",
    )
    .bind(account_id)
    .bind(balance_date)
    .bind(balance_minor)
    .execute(&mut **tx)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn account_snapshot_update_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    snapshot_id: i64,
    balance_date: NaiveDate,
    balance_minor: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE account_balance_snapshots
        SET
            balance_date = ?,
            balance_minor = ?
        WHERE
            account_id = ?
            AND id = ?
        ",
    )
    .bind(balance_date)
    .bind(balance_minor)
    .bind(account_id)
    .bind(snapshot_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn account_snapshot_delete_many_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    snapshot_ids: &[i64],
) -> Result<u64, sqlx::Error> {
    if snapshot_ids.is_empty() {
        return Ok(0);
    }

    let mut qb =
        QueryBuilder::<Sqlite>::new("DELETE FROM account_balance_snapshots WHERE account_id = ");
    qb.push_bind(account_id);
    qb.push(" AND id IN (");
    {
        let mut separated = qb.separated(", ");
        for snapshot_id in snapshot_ids {
            separated.push_bind(snapshot_id);
        }
    }
    qb.push(")");

    let result = qb.build().execute(&mut **tx).await?;
    Ok(result.rows_affected())
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

pub async fn institution_exists(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists: Option<i64> = sqlx::query_scalar(
        r"
        SELECT
            id
        FROM
            institutions
        WHERE
            id = ?
        ",
    )
    .bind(institution_id)
    .fetch_optional(pool)
    .await?;

    Ok(exists.is_some())
}

pub async fn institution_name_exists(
    pool: &SqlitePool,
    name: &str,
    exclude_institution_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let mut qb = QueryBuilder::<Sqlite>::new(
        r"
        SELECT
            id
        FROM
            institutions
        WHERE
            name = ",
    );
    qb.push_bind(name);
    if let Some(exclude_id) = exclude_institution_id {
        qb.push(" AND id <> ");
        qb.push_bind(exclude_id);
    }
    qb.push(" LIMIT 1");

    let exists = qb.build_query_scalar::<i64>().fetch_optional(pool).await?;
    Ok(exists.is_some())
}

pub async fn institution_create(
    pool: &SqlitePool,
    name: &str,
) -> Result<rows::InstitutionRow, sqlx::Error> {
    let result = sqlx::query(
        r"
        INSERT INTO
            institutions (name)
        VALUES
            (?)
        ",
    )
    .bind(name)
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();

    let created = sqlx::query_as::<_, rows::InstitutionRow>(
        r"
        SELECT
            id,
            name
        FROM
            institutions
        WHERE
            id = ?
        ",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(created)
}

pub async fn institution_create_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    name: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r"
        INSERT INTO
            institutions (name)
        VALUES
            (?)
        ",
    )
    .bind(name)
    .execute(&mut **tx)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn institution_update(
    pool: &SqlitePool,
    institution_id: i64,
    name: &str,
) -> Result<Option<rows::InstitutionRow>, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE institutions
        SET
            name = ?
        WHERE
            id = ?
        ",
    )
    .bind(name)
    .bind(institution_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    let updated = sqlx::query_as::<_, rows::InstitutionRow>(
        r"
        SELECT
            id,
            name
        FROM
            institutions
        WHERE
            id = ?
        ",
    )
    .bind(institution_id)
    .fetch_optional(pool)
    .await?;

    Ok(updated)
}

pub async fn account_type_id_by_name(
    pool: &SqlitePool,
    type_name: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let id: Option<i64> = sqlx::query_scalar(
        r"
        SELECT
            id
        FROM
            account_types
        WHERE
            name = ?
        ",
    )
    .bind(type_name)
    .fetch_optional(pool)
    .await?;

    Ok(id)
}

pub async fn account_name_exists_in_institution(
    pool: &SqlitePool,
    institution_id: i64,
    name: &str,
    exclude_account_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let mut qb = QueryBuilder::<Sqlite>::new(
        r"
        SELECT
            id
        FROM
            accounts
        WHERE
            institution_id = ",
    );
    qb.push_bind(institution_id);
    qb.push(" AND name = ");
    qb.push_bind(name);
    if let Some(exclude_id) = exclude_account_id {
        qb.push(" AND id <> ");
        qb.push_bind(exclude_id);
    }
    qb.push(" LIMIT 1");

    let existing = qb.build_query_scalar::<i64>().fetch_optional(pool).await?;
    Ok(existing.is_some())
}

#[derive(Debug, Clone)]
pub struct AccountMutationInput {
    pub institution_id: i64,
    pub name: String,
    pub type_id: i64,
    pub currency_code: String,
    pub normal_balance_sign: i32,
    pub opened_date: Option<NaiveDate>,
}

pub async fn account_create(
    pool: &SqlitePool,
    input: &AccountMutationInput,
) -> Result<rows::AccountRow, sqlx::Error> {
    let result = sqlx::query(
        r"
        INSERT INTO
            accounts (
                name,
                institution_id,
                type_id,
                currency_code,
                normal_balance_sign,
                opened_date
            )
        VALUES
            (?, ?, ?, ?, ?, ?)
        ",
    )
    .bind(&input.name)
    .bind(input.institution_id)
    .bind(input.type_id)
    .bind(&input.currency_code)
    .bind(input.normal_balance_sign)
    .bind(input.opened_date)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();
    let created = sqlx::query_as::<_, rows::AccountRow>(
        r"
        SELECT
            id,
            name,
            institution_id,
            type_id,
            currency_code,
            normal_balance_sign,
            opened_date,
            closed_date,
            created_at,
            updated_at
        FROM
            accounts
        WHERE
            id = ?
        ",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(created)
}

pub async fn account_create_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    input: &AccountMutationInput,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r"
        INSERT INTO
            accounts (
                name,
                institution_id,
                type_id,
                currency_code,
                normal_balance_sign,
                opened_date
            )
        VALUES
            (?, ?, ?, ?, ?, ?)
        ",
    )
    .bind(&input.name)
    .bind(input.institution_id)
    .bind(input.type_id)
    .bind(&input.currency_code)
    .bind(input.normal_balance_sign)
    .bind(input.opened_date)
    .execute(&mut **tx)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn account_update(
    pool: &SqlitePool,
    account_id: i64,
    input: &AccountMutationInput,
) -> Result<Option<rows::AccountRow>, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE accounts
        SET
            institution_id = ?,
            name = ?,
            type_id = ?,
            currency_code = ?,
            normal_balance_sign = ?,
            opened_date = ?,
            updated_at = STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE
            id = ?
        ",
    )
    .bind(input.institution_id)
    .bind(&input.name)
    .bind(input.type_id)
    .bind(&input.currency_code)
    .bind(input.normal_balance_sign)
    .bind(input.opened_date)
    .bind(account_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    let updated = sqlx::query_as::<_, rows::AccountRow>(
        r"
        SELECT
            id,
            name,
            institution_id,
            type_id,
            currency_code,
            normal_balance_sign,
            opened_date,
            closed_date,
            created_at,
            updated_at
        FROM
            accounts
        WHERE
            id = ?
        ",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    Ok(updated)
}

pub async fn account_update_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    input: &AccountMutationInput,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE accounts
        SET
            institution_id = ?,
            name = ?,
            type_id = ?,
            currency_code = ?,
            normal_balance_sign = ?,
            opened_date = ?,
            updated_at = STRFTIME('%Y-%m-%dT%H:%M:%SZ', 'now')
        WHERE
            id = ?
        ",
    )
    .bind(input.institution_id)
    .bind(&input.name)
    .bind(input.type_id)
    .bind(&input.currency_code)
    .bind(input.normal_balance_sign)
    .bind(input.opened_date)
    .bind(account_id)
    .execute(&mut **tx)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn account_delete_preview(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Option<AccountDeletePreviewRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, AccountDeletePreviewRow>(
        r"
        SELECT
            a.id,
            a.name,
            i.name AS institution_name,
            COUNT(s.id) AS snapshot_count
        FROM
            accounts AS a
            INNER JOIN institutions AS i ON i.id = a.institution_id
            LEFT JOIN account_balance_snapshots AS s ON s.account_id = a.id
        WHERE
            a.id = ?
        GROUP BY
            a.id,
            a.name,
            i.name
        ",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn institution_accounts_delete_preview(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<Vec<InstitutionAccountDeletePreviewRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, InstitutionAccountDeletePreviewRow>(
        r"
        SELECT
            a.id,
            a.name,
            COUNT(s.id) AS snapshot_count
        FROM
            accounts AS a
            LEFT JOIN account_balance_snapshots AS s ON s.account_id = a.id
        WHERE
            a.institution_id = ?
        GROUP BY
            a.id,
            a.name
        ORDER BY
            a.name ASC
        ",
    )
    .bind(institution_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn account_delete(pool: &SqlitePool, account_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        DELETE FROM accounts
        WHERE
            id = ?
        ",
    )
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn institution_delete(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        DELETE FROM institutions
        WHERE
            id = ?
        ",
    )
    .bind(institution_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
