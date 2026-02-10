use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use chrono::{Duration, NaiveDate, Utc};
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use tauri::State;

use crate::state::AppState;
use crate::{db, db::AccountListRow};

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ApiError {
    #[error("Database error")]
    Db,
    #[error("Not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum AccountTypeName {
    Current,
    Savings,
    CreditCard,
    Isa,
    Investment,
    Pension,
    Cash,
    Loan,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum ActivityPeriod {
    #[serde(rename = "1W")]
    P1W,
    #[serde(rename = "1M")]
    P1M,
    #[serde(rename = "3M")]
    P3M,
    #[serde(rename = "6M")]
    P6M,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
pub enum BalanceOverTimePeriod {
    #[serde(rename = "1M")]
    P1M,
    #[serde(rename = "6M")]
    P6M,
    #[serde(rename = "1Y")]
    P1Y,
    #[serde(rename = "MAX")]
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivityDataDto {
    pub values: Vec<Option<i64>>, // minor units (e.g. pennies)
    pub delta_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstitutionDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccountTypeDto {
    pub id: i64,
    pub name: AccountTypeName,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccountDto {
    pub id: i64,
    pub name: String,
    pub institution: InstitutionDto,
    pub account_type: AccountTypeDto,
    pub currency_code: String,
    pub normal_balance_sign: i32, // {-1, 1}
    pub opened_date: Option<NaiveDate>,
    pub closed_date: Option<NaiveDate>,
    pub first_snapshot_date: NaiveDate,
    pub latest_snapshot_date: NaiveDate,
    pub latest_balance_minor: i64,
    pub activity_by_period: BTreeMap<ActivityPeriod, ActivityDataDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardAllocationDto {
    pub account_type: AccountTypeName,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardBalancePointDto {
    pub date: NaiveDate,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccountBalanceSnapshotDto {
    pub id: i64,
    pub date: NaiveDate,
    pub balance_minor: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BalancePointDto {
    pub date: NaiveDate,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardDto {
    pub total_balance_minor: i64,
    pub change_vs_last_month_pct: f64,
    pub monthly_yield_minor: i64,
    pub active_accounts: u32,
    pub allocation_by_type: Vec<DashboardAllocationDto>,
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_list(state: State<'_, AppState>) -> Result<Vec<AccountDto>, ApiError> {
    let pool = &state.pool;
    let today = Utc::now().date_naive();

    let accounts = db::accounts_list_full(pool)
        .await
        .map_err(|_| ApiError::Db)?;
    let account_ids = accounts.iter().map(|a| a.id).collect::<Vec<_>>();

    // Longest period shown in the UI is 6M (180 points). We always build that once and slice.
    let full_points: usize = 180;
    let full_start = today - Duration::days(full_points as i64 - 1);

    let snapshots = db::snapshots_for_accounts_between(pool, &account_ids, full_start, today)
        .await
        .map_err(|_| ApiError::Db)?;
    let last_before = db::last_snapshots_before(pool, &account_ids, full_start)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut snapshots_by_account: HashMap<i64, HashMap<NaiveDate, i64>> = HashMap::new();
    for s in snapshots {
        snapshots_by_account
            .entry(s.account_id)
            .or_default()
            .insert(s.balance_date, s.balance_minor);
    }

    let initial_before: HashMap<i64, i64> = last_before
        .into_iter()
        .map(|s| (s.account_id, s.balance_minor))
        .collect();

    let mut out = Vec::with_capacity(accounts.len());
    let empty_dates: HashMap<NaiveDate, i64> = HashMap::new();
    for a in accounts {
        let account_type_name = account_type_from_db(&a.type_name)?;

        let institution = InstitutionDto {
            id: a.institution_id,
            name: a.institution_name,
        };
        let account_type = AccountTypeDto {
            id: a.type_id,
            name: account_type_name,
        };

        let first_snapshot_date = a.first_snapshot_date.unwrap_or(today);
        let latest_snapshot_date = a.latest_snapshot_date.unwrap_or(today);
        let latest_balance_minor = a.latest_balance_minor.unwrap_or(0);

        let date_map = snapshots_by_account.get(&a.id).unwrap_or(&empty_dates);
        let seed_before = initial_before.get(&a.id).copied();
        let series_180 = filled_values_for_period(date_map, seed_before, full_start, full_points);

        let activity_by_period: BTreeMap<_, _> = [
            (ActivityPeriod::P1W, 7),
            (ActivityPeriod::P1M, 30),
            (ActivityPeriod::P3M, 90),
            (ActivityPeriod::P6M, 180),
        ]
        .into_iter()
        .map(|(period, points)| {
            let values = series_180[series_180.len().saturating_sub(points)..].to_vec();
            let delta_minor = match (
                values.iter().find_map(|&v| v),
                values.iter().rev().find_map(|&v| v),
            ) {
                (Some(first), Some(last)) => last - first,
                _ => 0,
            };

            (
                period,
                ActivityDataDto {
                    values,
                    delta_minor,
                },
            )
        })
        .collect();

        out.push(AccountDto {
            id: a.id,
            name: a.name,
            institution,
            account_type,
            currency_code: a.currency_code,
            normal_balance_sign: a.normal_balance_sign,
            opened_date: a.opened_date,
            closed_date: a.closed_date,
            first_snapshot_date,
            latest_snapshot_date,
            latest_balance_minor,
            activity_by_period,
        });
    }

    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_get(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDto, ApiError> {
    let pool = &state.pool;
    let today = Utc::now().date_naive();

    let Some(a) = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let account_type_name = account_type_from_db(&a.type_name)?;

    let institution = InstitutionDto {
        id: a.institution_id,
        name: a.institution_name,
    };
    let account_type = AccountTypeDto {
        id: a.type_id,
        name: account_type_name,
    };

    let first_snapshot_date = a.first_snapshot_date.unwrap_or(today);
    let latest_snapshot_date = a.latest_snapshot_date.unwrap_or(today);
    let latest_balance_minor = a.latest_balance_minor.unwrap_or(0);

    // Build activity series exactly like `accounts_list` (seed from last snapshot before range).
    let full_points: usize = 180;
    let full_start = today - Duration::days(full_points as i64 - 1);

    let account_ids = vec![account_id];
    let snapshots = db::snapshots_for_accounts_between(pool, &account_ids, full_start, today)
        .await
        .map_err(|_| ApiError::Db)?;
    let last_before = db::last_snapshots_before(pool, &account_ids, full_start)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut date_map: HashMap<NaiveDate, i64> = HashMap::new();
    for s in snapshots {
        date_map.insert(s.balance_date, s.balance_minor);
    }

    let seed_before = last_before.first().map(|s| s.balance_minor);
    let series_180 = filled_values_for_period(&date_map, seed_before, full_start, full_points);

    let activity_by_period: BTreeMap<_, _> = [
        (ActivityPeriod::P1W, 7),
        (ActivityPeriod::P1M, 30),
        (ActivityPeriod::P3M, 90),
        (ActivityPeriod::P6M, 180),
    ]
    .into_iter()
    .map(|(period, points)| {
        let values = series_180[series_180.len().saturating_sub(points)..].to_vec();
        let delta_minor = match (
            values.iter().find_map(|&v| v),
            values.iter().rev().find_map(|&v| v),
        ) {
            (Some(first), Some(last)) => last - first,
            _ => 0,
        };

        (
            period,
            ActivityDataDto {
                values,
                delta_minor,
            },
        )
    })
    .collect();

    Ok(AccountDto {
        id: a.id,
        name: a.name,
        institution,
        account_type,
        currency_code: a.currency_code,
        normal_balance_sign: a.normal_balance_sign,
        opened_date: a.opened_date,
        closed_date: a.closed_date,
        first_snapshot_date,
        latest_snapshot_date,
        latest_balance_minor,
        activity_by_period,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshots_list(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<AccountBalanceSnapshotDto>, ApiError> {
    let pool = &state.pool;

    // Ensure account exists for consistent NotFound behavior.
    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    let rows = db::snapshots_for_account(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| AccountBalanceSnapshotDto {
            id: r.id,
            date: r.balance_date,
            balance_minor: r.balance_minor,
            created_at: r.created_at,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn account_balance_over_time(
    state: State<'_, AppState>,
    account_id: i64,
    period: BalanceOverTimePeriod,
) -> Result<Vec<BalancePointDto>, ApiError> {
    let today = Utc::now().date_naive();
    let pool = &state.pool;

    // Ensure account exists.
    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    let Some(earliest) = db::earliest_snapshot_date_for_account(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Ok(Vec::new());
    };

    let computed_start = match period {
        BalanceOverTimePeriod::P1M => today - Duration::days(30 - 1),
        BalanceOverTimePeriod::P6M => today - Duration::days(183 - 1),
        BalanceOverTimePeriod::P1Y => today - Duration::days(365 - 1),
        BalanceOverTimePeriod::Max => earliest,
    };

    let start = std::cmp::max(computed_start, earliest);
    if today < start {
        return Ok(Vec::new());
    }

    let points = (today - start).num_days() as usize + 1;
    let account_ids = vec![account_id];

    let snapshots = db::snapshots_for_accounts_between(pool, &account_ids, start, today)
        .await
        .map_err(|_| ApiError::Db)?;
    let last_before = db::last_snapshots_before(pool, &account_ids, start)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut date_map: HashMap<NaiveDate, i64> = HashMap::new();
    for s in snapshots {
        date_map.insert(s.balance_date, s.balance_minor);
    }

    let seed_before = last_before.first().map(|s| s.balance_minor);
    let series = filled_values_for_period(&date_map, seed_before, start, points);

    Ok(series
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            let date = start + Duration::days(i as i64);
            BalancePointDto {
                date,
                balance_minor: v.unwrap_or(0),
            }
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn dashboard_get(state: State<'_, AppState>) -> Result<DashboardDto, ApiError> {
    let pool = &state.pool;
    let today = Utc::now().date_naive();

    let accounts = db::accounts_list_full(pool)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut total_balance_minor: i64 = 0;
    let mut active_accounts: u32 = 0;
    let mut allocation: BTreeMap<AccountTypeName, i64> = BTreeMap::new();

    for a in &accounts {
        let account_type_name = account_type_from_db(&a.type_name)?;
        let latest_minor = a.latest_balance_minor.unwrap_or(0);
        total_balance_minor += latest_minor;
        if latest_minor != 0 {
            active_accounts += 1;
        }
        *allocation.entry(account_type_name).or_insert(0) += latest_minor;
    }

    // Pie chart allocations should be non-negative; exclude net-negative/zero groups (e.g. credit cards).
    let allocation_by_type = allocation
        .into_iter()
        .filter(|(_kind, balance_minor)| *balance_minor > 0)
        .map(|(account_type, balance_minor)| DashboardAllocationDto {
            account_type,
            balance_minor,
        })
        .collect::<Vec<_>>();

    // Compute monthly metrics using a minimal 31-point (30-day) series.
    let start_30d = today - Duration::days(31 - 1);
    let balance_30d = total_balance_over_time(pool, &accounts, start_30d, today).await?;

    let last_minor = balance_30d
        .last()
        .map_or(total_balance_minor, |p| p.balance_minor);

    let month_ago_minor = balance_30d
        .get(balance_30d.len().saturating_sub(31))
        .map_or(last_minor, |p| p.balance_minor);

    let monthly_yield_minor = last_minor - month_ago_minor;
    let change_vs_last_month_pct = if month_ago_minor != 0 {
        (monthly_yield_minor as f64) / (month_ago_minor as f64) * 100.0
    } else {
        0.0
    };

    Ok(DashboardDto {
        total_balance_minor,
        change_vs_last_month_pct,
        monthly_yield_minor,
        active_accounts,
        allocation_by_type,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn dashboard_balance_over_time(
    state: State<'_, AppState>,
    period: BalanceOverTimePeriod,
) -> Result<Vec<DashboardBalancePointDto>, ApiError> {
    let today = Utc::now().date_naive();
    let pool = &state.pool;
    let accounts = db::accounts_list_full(pool)
        .await
        .map_err(|_| ApiError::Db)?;

    let start = match period {
        BalanceOverTimePeriod::P1M => today - Duration::days(30 - 1),
        BalanceOverTimePeriod::P6M => today - Duration::days(183 - 1),
        BalanceOverTimePeriod::P1Y => today - Duration::days(365 - 1),
        BalanceOverTimePeriod::Max => match db::earliest_snapshot_date(pool)
            .await
            .map_err(|_| ApiError::Db)?
        {
            Some(s) => s,
            None => today,
        },
    };

    total_balance_over_time(pool, &accounts, start, today).await
}

fn account_type_from_db(name: &str) -> Result<AccountTypeName, ApiError> {
    match name {
        "current" => Ok(AccountTypeName::Current),
        "savings" => Ok(AccountTypeName::Savings),
        "credit_card" => Ok(AccountTypeName::CreditCard),
        "isa" => Ok(AccountTypeName::Isa),
        "investment" => Ok(AccountTypeName::Investment),
        "pension" => Ok(AccountTypeName::Pension),
        "cash" => Ok(AccountTypeName::Cash),
        "loan" => Ok(AccountTypeName::Loan),
        other => Err(ApiError::Validation(format!(
            "unknown account type name in DB: {other}"
        ))),
    }
}

fn filled_values_for_period(
    snapshots_by_date: &HashMap<NaiveDate, i64>,
    initial_before: Option<i64>,
    period_start: NaiveDate,
    points: usize,
) -> Vec<Option<i64>> {
    let mut last = initial_before;
    (0..points)
        .map(|i| {
            let date = period_start + Duration::days(i as i64);
            last = snapshots_by_date.get(&date).copied().or(last);
            last
        })
        .collect()
}

async fn total_balance_over_time(
    pool: &SqlitePool,
    accounts: &[AccountListRow],
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<DashboardBalancePointDto>, ApiError> {
    if end < start {
        return Ok(Vec::new());
    }

    let points = (end - start).num_days() as usize + 1;
    let account_ids = accounts.iter().map(|a| a.id).collect::<Vec<_>>();

    let snapshots = db::snapshots_for_accounts_between(pool, &account_ids, start, end)
        .await
        .map_err(|_| ApiError::Db)?;
    let last_before = db::last_snapshots_before(pool, &account_ids, start)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut snapshots_by_account: HashMap<i64, HashMap<NaiveDate, i64>> = HashMap::new();
    for s in snapshots {
        snapshots_by_account
            .entry(s.account_id)
            .or_default()
            .insert(s.balance_date, s.balance_minor);
    }

    let initial_before: HashMap<i64, i64> = last_before
        .into_iter()
        .map(|s| (s.account_id, s.balance_minor))
        .collect();

    // Forward-fill each account, then sum per day.
    let mut totals = vec![0i64; points];
    let empty_dates: HashMap<NaiveDate, i64> = HashMap::new();
    for a in accounts {
        let date_map = snapshots_by_account.get(&a.id).unwrap_or(&empty_dates);
        let seed_before = initial_before.get(&a.id).copied();
        let series = filled_values_for_period(date_map, seed_before, start, points);
        for (i, v) in series.into_iter().enumerate() {
            if let Some(minor) = v {
                totals[i] += minor;
            }
        }
    }

    let out: Vec<_> = totals
        .into_iter()
        .enumerate()
        .map(|(i, balance_minor)| {
            let date = start + Duration::days(i as i64);
            DashboardBalancePointDto {
                date,
                balance_minor,
            }
        })
        .collect();

    Ok(out)
}

pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    use tauri_specta::{collect_commands, Builder};

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        accounts_list,
        accounts_get,
        account_snapshots_list,
        account_balance_over_time,
        dashboard_get,
        dashboard_balance_over_time,
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
