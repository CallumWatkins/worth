use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use chrono::{Duration, NaiveDate, Utc};
use garde::Validate;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap, HashSet};
use tauri::State;

use crate::contracts::{
    AccountTypeName, AccountUpsertInput, InstitutionRef, InstitutionUpsertInput,
};
use crate::state::AppState;
use crate::{db, db::AccountListRow};

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ApiError {
    #[error("Database error")]
    Db,
    #[error("Not found")]
    NotFound,
    #[error("Validation error")]
    Validation(Vec<ValidationIssue>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
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
pub struct InstitutionDetailDto {
    pub id: i64,
    pub name: String,
    pub accounts: Vec<AccountDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstitutionSummaryDto {
    pub id: i64,
    pub name: String,
    pub account_count: u32,
    pub empty_account_count: u32,
    pub account_types: Vec<AccountTypeName>,
    pub total_balance_minor: i64,
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
pub struct CreatedIdDto {
    pub id: i64,
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
    pub active_institutions: u32,
    pub allocation_by_type: Vec<DashboardAllocationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SearchResultDto {
    Account {
        id: i64,
        name: String,
        account_type: AccountTypeName,
        institution_name: String,
    },
    Institution {
        id: i64,
        name: String,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_list(state: State<'_, AppState>) -> Result<Vec<AccountDto>, ApiError> {
    let pool = &state.pool;

    let accounts = db::accounts_list_full(pool)
        .await
        .map_err(|_| ApiError::Db)?;
    build_account_dtos(pool, accounts).await
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_list(
    state: State<'_, AppState>,
) -> Result<Vec<InstitutionSummaryDto>, ApiError> {
    let pool = &state.pool;

    let summary_rows = db::institutions_list_summary(pool)
        .await
        .map_err(|_| ApiError::Db)?;
    let type_rows = db::institutions_account_types(pool)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut types_by_institution: HashMap<i64, Vec<AccountTypeName>> =
        HashMap::with_capacity(summary_rows.len());
    for row in type_rows {
        let account_type_name = account_type_from_db(&row.type_name)?;
        types_by_institution
            .entry(row.institution_id)
            .or_default()
            .push(account_type_name);
    }

    let mut out = Vec::with_capacity(summary_rows.len());
    for row in summary_rows {
        out.push(InstitutionSummaryDto {
            id: row.id,
            name: row.name,
            account_count: u32::try_from(row.account_count)
                .expect("account count should fit in u32"),
            empty_account_count: u32::try_from(row.empty_account_count)
                .expect("empty account count should fit in u32"),
            account_types: types_by_institution.remove(&row.id).unwrap_or_default(),
            total_balance_minor: row.total_balance_minor,
        });
    }

    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResultDto>, ApiError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let pool = &state.pool;
    let rows = db::search_global(pool, query)
        .await
        .map_err(|_| ApiError::Db)?;

    rows.into_iter()
        .map(|row| match row {
            db::GlobalSearchRow::Account {
                id,
                name,
                type_name,
                institution_name,
            } => Ok(SearchResultDto::Account {
                id,
                name,
                account_type: account_type_from_db(&type_name)?,
                institution_name,
            }),
            db::GlobalSearchRow::Institution { id, name } => {
                Ok(SearchResultDto::Institution { id, name })
            }
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_get(
    state: State<'_, AppState>,
    institution_id: i64,
) -> Result<InstitutionDetailDto, ApiError> {
    let pool = &state.pool;
    institution_detail_by_id(pool, institution_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_create(
    state: State<'_, AppState>,
    input: InstitutionUpsertInput,
) -> Result<CreatedIdDto, ApiError> {
    let pool = &state.pool;
    let validated = validate_institution_upsert(pool, &input, None).await?;

    let created = db::institution_create(pool, &validated.name)
        .await
        .map_err(map_institution_write_error)?;

    Ok(CreatedIdDto { id: created.id })
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_update(
    state: State<'_, AppState>,
    institution_id: i64,
    input: InstitutionUpsertInput,
) -> Result<(), ApiError> {
    let pool = &state.pool;
    let validated = validate_institution_upsert(pool, &input, Some(institution_id)).await?;

    let updated = db::institution_update(pool, institution_id, &validated.name)
        .await
        .map_err(map_institution_write_error)?;

    if updated.is_none() {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_get(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDto, ApiError> {
    let pool = &state.pool;
    account_dto_by_id(pool, account_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_create(
    state: State<'_, AppState>,
    input: AccountUpsertInput,
) -> Result<CreatedIdDto, ApiError> {
    let pool = &state.pool;
    let validated = validate_account_upsert(pool, &input, None).await?;

    let account_id = match &validated.institution {
        ValidatedInstitutionRef::Existing { id } => {
            let mutation = db::AccountMutationInput {
                institution_id: *id,
                name: validated.name.clone(),
                type_id: validated.type_id,
                currency_code: validated.currency_code.clone(),
                normal_balance_sign: validated.normal_balance_sign,
                opened_date: validated.opened_date,
            };

            db::account_create(pool, &mutation)
                .await
                .map_err(map_account_write_error)?
                .id
        }
        ValidatedInstitutionRef::New { name } => {
            let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;

            let institution_id = db::institution_create_tx(&mut tx, name)
                .await
                .map_err(map_institution_write_error)?;

            let mutation = db::AccountMutationInput {
                institution_id,
                name: validated.name.clone(),
                type_id: validated.type_id,
                currency_code: validated.currency_code.clone(),
                normal_balance_sign: validated.normal_balance_sign,
                opened_date: validated.opened_date,
            };

            let account_id = db::account_create_tx(&mut tx, &mutation)
                .await
                .map_err(map_account_write_error)?;

            tx.commit().await.map_err(|_| ApiError::Db)?;
            account_id
        }
    };

    Ok(CreatedIdDto { id: account_id })
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_update(
    state: State<'_, AppState>,
    account_id: i64,
    input: AccountUpsertInput,
) -> Result<(), ApiError> {
    let pool = &state.pool;

    // Keep behavior explicit before we potentially create a new institution.
    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    let validated = validate_account_upsert(pool, &input, Some(account_id)).await?;

    match &validated.institution {
        ValidatedInstitutionRef::Existing { id } => {
            let mutation = db::AccountMutationInput {
                institution_id: *id,
                name: validated.name.clone(),
                type_id: validated.type_id,
                currency_code: validated.currency_code.clone(),
                normal_balance_sign: validated.normal_balance_sign,
                opened_date: validated.opened_date,
            };

            let updated = db::account_update(pool, account_id, &mutation)
                .await
                .map_err(map_account_write_error)?;
            if updated.is_none() {
                return Err(ApiError::NotFound);
            }
        }
        ValidatedInstitutionRef::New { name } => {
            let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;
            let institution_id = db::institution_create_tx(&mut tx, name)
                .await
                .map_err(map_institution_write_error)?;
            let mutation = db::AccountMutationInput {
                institution_id,
                name: validated.name.clone(),
                type_id: validated.type_id,
                currency_code: validated.currency_code.clone(),
                normal_balance_sign: validated.normal_balance_sign,
                opened_date: validated.opened_date,
            };

            let updated = db::account_update_tx(&mut tx, account_id, &mutation)
                .await
                .map_err(map_account_write_error)?;
            if !updated {
                return Err(ApiError::NotFound);
            }

            tx.commit().await.map_err(|_| ApiError::Db)?;
        }
    }

    Ok(())
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
    let mut active_institution_ids: HashSet<i64> = HashSet::new();
    let mut allocation: BTreeMap<AccountTypeName, i64> = BTreeMap::new();

    for a in &accounts {
        let account_type_name = account_type_from_db(&a.type_name)?;
        let latest_minor = a.latest_balance_minor.unwrap_or(0);
        total_balance_minor += latest_minor;
        if latest_minor != 0 {
            active_accounts += 1;
            active_institution_ids.insert(a.institution_id);
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
        active_institutions: u32::try_from(active_institution_ids.len())
            .expect("active institution count should fit in u32"),
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

#[derive(Debug, Clone)]
struct ValidatedInstitutionUpsert {
    name: String,
}

#[derive(Debug, Clone)]
enum ValidatedInstitutionRef {
    Existing { id: i64 },
    New { name: String },
}

#[derive(Debug, Clone)]
struct ValidatedAccountUpsert {
    institution: ValidatedInstitutionRef,
    name: String,
    type_id: i64,
    currency_code: String,
    normal_balance_sign: i32,
    opened_date: Option<NaiveDate>,
}

async fn institution_detail_by_id(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<InstitutionDetailDto, ApiError> {
    let Some(institution) = db::institution_get(pool, institution_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let accounts = db::accounts_list_full_for_institution(pool, institution_id)
        .await
        .map_err(|_| ApiError::Db)?;
    let account_dtos = build_account_dtos(pool, accounts).await?;

    Ok(InstitutionDetailDto {
        id: institution.id,
        name: institution.name,
        accounts: account_dtos,
    })
}

async fn account_dto_by_id(pool: &SqlitePool, account_id: i64) -> Result<AccountDto, ApiError> {
    let Some(account_row) = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let mut dtos = build_account_dtos(pool, vec![account_row]).await?;
    dtos.pop().ok_or(ApiError::NotFound)
}

async fn validate_institution_upsert(
    pool: &SqlitePool,
    input: &InstitutionUpsertInput,
    exclude_institution_id: Option<i64>,
) -> Result<ValidatedInstitutionUpsert, ApiError> {
    let normalized = normalize_institution_upsert(input);
    let mut issues = validation_issues_from_garde_report(normalized.validate().err());

    if issues.is_empty() {
        let exists = db::institution_name_exists(pool, &normalized.name, exclude_institution_id)
            .await
            .map_err(|_| ApiError::Db)?;
        if exists {
            issues.push(validation_issue(
                "name",
                "An institution with this name already exists",
            ));
        }
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(ValidatedInstitutionUpsert {
        name: normalized.name,
    })
}

async fn validate_account_upsert(
    pool: &SqlitePool,
    input: &AccountUpsertInput,
    exclude_account_id: Option<i64>,
) -> Result<ValidatedAccountUpsert, ApiError> {
    let normalized = normalize_account_upsert(input);
    let mut issues = validation_issues_from_garde_report(normalized.validate().err());

    let account_type_db = account_type_to_db(normalized.account_type);
    let type_id = db::account_type_id_by_name(pool, account_type_db)
        .await
        .map_err(|_| ApiError::Db)?;
    if type_id.is_none() {
        issues.push(validation_issue("account_type", "Invalid account type"));
    }

    let institution = match &normalized.institution {
        InstitutionRef::Existing { id } => {
            let exists = db::institution_exists(pool, *id)
                .await
                .map_err(|_| ApiError::Db)?;
            if !exists {
                issues.push(validation_issue(
                    "institution.id",
                    "Institution does not exist",
                ));
            }
            ValidatedInstitutionRef::Existing { id: *id }
        }
        InstitutionRef::New { input } => {
            if issues.is_empty() {
                let exists = db::institution_name_exists(pool, &input.name, None)
                    .await
                    .map_err(|_| ApiError::Db)?;
                if exists {
                    issues.push(validation_issue(
                        "institution.input.name",
                        "An institution with this name already exists",
                    ));
                }
            }

            ValidatedInstitutionRef::New {
                name: input.name.clone(),
            }
        }
    };

    if issues.is_empty() {
        if let ValidatedInstitutionRef::Existing { id } = &institution {
            let duplicate = db::account_name_exists_in_institution(
                pool,
                *id,
                &normalized.name,
                exclude_account_id,
            )
            .await
            .map_err(|_| ApiError::Db)?;
            if duplicate {
                issues.push(validation_issue(
                    "name",
                    "An account with this name already exists for this institution",
                ));
            }
        }
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(ValidatedAccountUpsert {
        institution,
        name: normalized.name,
        type_id: type_id.expect("validated above"),
        currency_code: normalized.currency_code,
        normal_balance_sign: normalized.normal_balance_sign,
        opened_date: normalized.opened_date,
    })
}

fn validation_issue(field: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        field: field.to_string(),
        message: message.to_string(),
    }
}

fn validation_issues_from_garde_report(
    report: Option<garde::error::Report>,
) -> Vec<ValidationIssue> {
    let Some(report) = report else {
        return Vec::new();
    };

    report
        .iter()
        .map(|(path, error)| {
            let field = garde_path_to_field(path);
            validation_issue(&field, &error.to_string())
        })
        .collect()
}

fn garde_path_to_field(path: &garde::error::Path) -> String {
    let raw = path.to_string();
    if raw == "$" {
        return String::new();
    }
    if let Some(stripped) = raw.strip_prefix("$.") {
        return stripped.to_string();
    }
    raw
}

fn normalize_institution_upsert(input: &InstitutionUpsertInput) -> InstitutionUpsertInput {
    InstitutionUpsertInput {
        name: input.name.trim().to_string(),
    }
}

fn normalize_account_upsert(input: &AccountUpsertInput) -> AccountUpsertInput {
    AccountUpsertInput {
        institution: match &input.institution {
            InstitutionRef::Existing { id } => InstitutionRef::Existing { id: *id },
            InstitutionRef::New { input } => InstitutionRef::New {
                input: normalize_institution_upsert(input),
            },
        },
        name: input.name.trim().to_string(),
        account_type: input.account_type,
        currency_code: input.currency_code.trim().to_uppercase(),
        normal_balance_sign: input.normal_balance_sign,
        opened_date: input.opened_date,
    }
}

fn map_institution_write_error(error: sqlx::Error) -> ApiError {
    if is_unique_constraint(&error, "institutions.name") {
        return ApiError::Validation(vec![validation_issue(
            "name",
            "An institution with this name already exists",
        )]);
    }

    ApiError::Db
}

fn map_account_write_error(error: sqlx::Error) -> ApiError {
    if is_unique_constraint(&error, "accounts.institution_id, accounts.name") {
        return ApiError::Validation(vec![validation_issue(
            "name",
            "An account with this name already exists for this institution",
        )]);
    }

    ApiError::Db
}

fn is_unique_constraint(error: &sqlx::Error, needle: &str) -> bool {
    let sqlx::Error::Database(db_error) = error else {
        return false;
    };
    db_error.message().contains("UNIQUE constraint failed") && db_error.message().contains(needle)
}

async fn build_account_dtos(
    pool: &SqlitePool,
    accounts: Vec<AccountListRow>,
) -> Result<Vec<AccountDto>, ApiError> {
    let today = Utc::now().date_naive();
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
        other => Err(ApiError::Validation(vec![validation_issue(
            "account_type",
            &format!("unknown account type name in DB: {other}"),
        )])),
    }
}

fn account_type_to_db(name: AccountTypeName) -> &'static str {
    match name {
        AccountTypeName::Current => "current",
        AccountTypeName::Savings => "savings",
        AccountTypeName::CreditCard => "credit_card",
        AccountTypeName::Isa => "isa",
        AccountTypeName::Investment => "investment",
        AccountTypeName::Pension => "pension",
        AccountTypeName::Cash => "cash",
        AccountTypeName::Loan => "loan",
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

fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    use tauri_specta::{collect_commands, Builder};

    Builder::<tauri::Wry>::new().commands(collect_commands![
        accounts_list,
        accounts_create,
        accounts_update,
        institutions_list,
        institutions_create,
        institutions_update,
        institutions_get,
        accounts_get,
        account_snapshots_list,
        account_balance_over_time,
        dashboard_get,
        dashboard_balance_over_time,
        search,
    ])
}

pub fn export_bindings_to_app_generated() -> anyhow::Result<()> {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    use std::path::PathBuf;

    let bindings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("app")
        .join("generated")
        .join("bindings.ts");

    specta_builder()
        .export(
            Typescript::default()
                .header("// Generated file, update with `bun run contracts:gen`.")
                .bigint(BigIntExportBehavior::Number),
            bindings_path,
        )
        .map_err(|error| anyhow::anyhow!("Failed to export typescript bindings: {error}"))
}

pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    specta_builder().invoke_handler()
}
