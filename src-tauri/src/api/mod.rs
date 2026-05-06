use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use chrono::{Duration, Local, NaiveDate, Utc};
use garde::Validate;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap, HashSet};
use tauri::State;

use crate::contracts::{
    AccountSnapshotUpdateInput, AccountSnapshotsCreateInput, AccountSnapshotsDeleteInput,
    AccountTypeName, AccountUpsertInput, AppLocaleCode, AppSettingsUpdateInput, CurrencyCode,
    InstitutionRef, InstitutionUpsertInput, ThemePreference,
};
use crate::imports::snapshots::{
    SnapshotImportCommitDto, SnapshotImportInspectionDto, SnapshotImportOptionsInput,
    SnapshotImportPlanningContext, SnapshotImportPreviewAction, SnapshotImportPreviewDto,
    SnapshotImportSourceInput, SnapshotImportValidationIssue,
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
    /// Form field path associated with the validation issue.
    ///
    /// Uses dot/index notation for nested fields, for example
    /// `institution.input.name` or `snapshots.0.date`.
    pub field: String,
    /// User-facing validation message shown in the app UI.
    pub message: String,
    /// Message that is safe to share as telemetry.
    ///
    /// This should either match `message` when it contains no sensitive data, contain
    /// a redacted/generalized alternative, or be `None` to suppress the validation
    /// message from telemetry entirely.
    pub telemetry_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppSettingsDto {
    pub analytics_enabled: bool,
    pub default_display_currency_code: CurrencyCode,
    pub display_locale: AppLocaleCode,
    pub theme: ThemePreference,
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
    pub currency_code: CurrencyCode,
    pub normal_balance_sign: i32, // {-1, 1}
    pub opened_date: Option<NaiveDate>,
    pub closed_date: Option<NaiveDate>,
    pub first_snapshot_date: Option<NaiveDate>,
    pub latest_snapshot_date: Option<NaiveDate>,
    pub latest_balance_minor: i64,
    pub monthly_change_minor: i64,
    pub activity_by_period: BTreeMap<ActivityPeriod, ActivityDataDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccountDeletePreviewDto {
    pub id: i64,
    pub name: String,
    pub institution_name: String,
    pub snapshot_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstitutionDeletePreviewAccountDto {
    pub id: i64,
    pub name: String,
    pub snapshot_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstitutionDeletePreviewDto {
    pub institution: InstitutionDto,
    pub accounts: Vec<InstitutionDeletePreviewAccountDto>,
    pub total_snapshots: u32,
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
    pub total_accounts: u32,
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
pub async fn settings_get(state: State<'_, AppState>) -> Result<AppSettingsDto, ApiError> {
    let pool = &state.pool;
    let row = db::app_settings_get(pool).await.map_err(|_| ApiError::Db)?;
    app_settings_dto_from_row(row)
}

#[tauri::command]
#[specta::specta]
pub async fn settings_update(
    state: State<'_, AppState>,
    input: AppSettingsUpdateInput,
) -> Result<AppSettingsDto, ApiError> {
    let pool = &state.pool;
    let mutation = db::AppSettingsMutationInput {
        analytics_enabled: input.analytics_enabled,
        default_display_currency_code: input
            .default_display_currency_code
            .map(|currency| currency.as_str().to_owned()),
        display_locale: input
            .display_locale
            .map(|locale| locale.as_str().to_owned()),
        theme: input.theme.map(|theme| theme.as_str().to_owned()),
    };

    let row = db::app_settings_update(pool, &mutation)
        .await
        .map_err(|_| ApiError::Db)?;
    app_settings_dto_from_row(row)
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
        let account_type_name = row.type_name.parse().map_err(|_| ApiError::Db)?;
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
                account_type: type_name.parse().map_err(|_| ApiError::Db)?,
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
pub async fn accounts_delete_preview(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDeletePreviewDto, ApiError> {
    let pool = &state.pool;
    let Some(row) = db::account_delete_preview(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    Ok(AccountDeletePreviewDto {
        id: row.id,
        name: row.name,
        institution_name: row.institution_name,
        snapshot_count: u32::try_from(row.snapshot_count)
            .expect("snapshot count should fit in u32"),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_delete(state: State<'_, AppState>, account_id: i64) -> Result<(), ApiError> {
    let pool = &state.pool;
    let deleted = db::account_delete(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(())
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
                closed_date: validated.closed_date,
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
                closed_date: validated.closed_date,
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
                closed_date: validated.closed_date,
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
                closed_date: validated.closed_date,
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
pub async fn account_snapshots_create(
    state: State<'_, AppState>,
    account_id: i64,
    input: AccountSnapshotsCreateInput,
) -> Result<(), ApiError> {
    let pool = &state.pool;

    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    validate_account_snapshots_create(pool, account_id, &input).await?;
    let snapshot_dates = input
        .snapshots
        .iter()
        .map(|snapshot| snapshot.date)
        .collect::<Vec<_>>();
    let existing_by_date = db::snapshots_for_account_dates(pool, account_id, &snapshot_dates)
        .await
        .map_err(|_| ApiError::Db)?
        .into_iter()
        .map(|row| (row.balance_date, row.id))
        .collect::<HashMap<_, _>>();

    let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;
    for snapshot in input.snapshots {
        if let Some(existing_id) = existing_by_date.get(&snapshot.date) {
            db::account_snapshot_update_tx(
                &mut tx,
                account_id,
                *existing_id,
                snapshot.date,
                snapshot.balance_minor,
            )
            .await
            .map_err(map_account_snapshot_write_error)?;
            continue;
        }

        db::account_snapshot_create_tx(&mut tx, account_id, snapshot.date, snapshot.balance_minor)
            .await
            .map_err(map_account_snapshot_write_error)?;
    }

    tx.commit().await.map_err(|_| ApiError::Db)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshot_update(
    state: State<'_, AppState>,
    account_id: i64,
    snapshot_id: i64,
    input: AccountSnapshotUpdateInput,
) -> Result<(), ApiError> {
    let pool = &state.pool;

    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    let current = db::account_snapshot_get(pool, account_id, snapshot_id)
        .await
        .map_err(|_| ApiError::Db)?;
    if current.is_none() {
        return Err(ApiError::NotFound);
    }

    validate_account_snapshot_update(pool, account_id, snapshot_id, &input).await?;
    let conflicting = db::snapshots_for_account_dates(pool, account_id, &[input.date])
        .await
        .map_err(|_| ApiError::Db)?
        .into_iter()
        .find(|snapshot| snapshot.id != snapshot_id);

    let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;
    if let Some(conflicting) = conflicting {
        db::account_snapshot_delete_many_tx(&mut tx, account_id, &[conflicting.id])
            .await
            .map_err(|_| ApiError::Db)?;
    }

    let updated = db::account_snapshot_update_tx(
        &mut tx,
        account_id,
        snapshot_id,
        input.date,
        input.balance_minor,
    )
    .await
    .map_err(map_account_snapshot_write_error)?;
    if !updated {
        return Err(ApiError::NotFound);
    }

    tx.commit().await.map_err(|_| ApiError::Db)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshots_delete(
    state: State<'_, AppState>,
    account_id: i64,
    input: AccountSnapshotsDeleteInput,
) -> Result<(), ApiError> {
    let pool = &state.pool;

    let exists = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
        .is_some();
    if !exists {
        return Err(ApiError::NotFound);
    }

    let snapshot_ids = validate_account_snapshots_delete(&input)?;

    let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;
    let deleted = db::account_snapshot_delete_many_tx(&mut tx, account_id, &snapshot_ids)
        .await
        .map_err(|_| ApiError::Db)?;
    if deleted != snapshot_ids.len() as u64 {
        return Err(ApiError::NotFound);
    }

    tx.commit().await.map_err(|_| ApiError::Db)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshot_import_inspect(
    input: SnapshotImportSourceInput,
) -> Result<SnapshotImportInspectionDto, ApiError> {
    crate::imports::snapshots::inspect_source(&input).map_err(map_snapshot_import_validation)
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshot_import_preview(
    state: State<'_, AppState>,
    account_id: i64,
    input: SnapshotImportSourceInput,
    options: SnapshotImportOptionsInput,
) -> Result<SnapshotImportPreviewDto, ApiError> {
    account_snapshot_import_preview_with_today(
        &state.pool,
        account_id,
        input,
        options,
        Local::now().date_naive(),
    )
    .await
}

async fn account_snapshot_import_preview_with_today(
    pool: &SqlitePool,
    account_id: i64,
    input: SnapshotImportSourceInput,
    options: SnapshotImportOptionsInput,
    today: NaiveDate,
) -> Result<SnapshotImportPreviewDto, ApiError> {
    let Some(account) = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let existing_snapshots = db::snapshots_for_account(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?;
    let plan = crate::imports::snapshots::plan_import(
        &input,
        &options,
        &existing_snapshots,
        SnapshotImportPlanningContext {
            account_opened_date: account.opened_date,
            account_closed_date: account.closed_date,
            today,
        },
    )
    .map_err(map_snapshot_import_validation)?;

    Ok(plan.preview)
}

#[tauri::command]
#[specta::specta]
pub async fn account_snapshot_import_commit(
    state: State<'_, AppState>,
    account_id: i64,
    input: SnapshotImportSourceInput,
    options: SnapshotImportOptionsInput,
) -> Result<SnapshotImportCommitDto, ApiError> {
    account_snapshot_import_commit_with_today(
        &state.pool,
        account_id,
        input,
        options,
        Local::now().date_naive(),
    )
    .await
}

async fn account_snapshot_import_commit_with_today(
    pool: &SqlitePool,
    account_id: i64,
    input: SnapshotImportSourceInput,
    options: SnapshotImportOptionsInput,
    today: NaiveDate,
) -> Result<SnapshotImportCommitDto, ApiError> {
    let Some(account) = db::account_get_full(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let existing_snapshots = db::snapshots_for_account(pool, account_id)
        .await
        .map_err(|_| ApiError::Db)?;
    let plan = crate::imports::snapshots::plan_import(
        &input,
        &options,
        &existing_snapshots,
        SnapshotImportPlanningContext {
            account_opened_date: account.opened_date,
            account_closed_date: account.closed_date,
            today,
        },
    )
    .map_err(map_snapshot_import_validation)?;

    if plan.preview.summary.invalid_count > 0 {
        return Err(ApiError::Validation(vec![validation_issue(
            "import",
            "Fix invalid rows before importing snapshots",
        )]));
    }

    if plan.preview.summary.overwrite_count > 0 && !options.overwrite_existing_confirmed {
        return Err(ApiError::Validation(vec![validation_issue(
            "import",
            "Confirm overwrite to continue",
        )]));
    }

    let mut tx = pool.begin().await.map_err(|_| ApiError::Db)?;
    let mut created_count = 0;
    let mut overwritten_count = 0;

    for write in plan.writes {
        match write.action {
            SnapshotImportPreviewAction::Create => {
                db::account_snapshot_create_tx(
                    &mut tx,
                    account_id,
                    write.date,
                    write.balance_minor,
                )
                .await
                .map_err(map_account_snapshot_write_error)?;
                created_count += 1;
            }
            SnapshotImportPreviewAction::Overwrite => {
                let Some(existing_snapshot_id) = write.existing_snapshot_id else {
                    return Err(ApiError::Db);
                };
                let updated = db::account_snapshot_update_tx(
                    &mut tx,
                    account_id,
                    existing_snapshot_id,
                    write.date,
                    write.balance_minor,
                )
                .await
                .map_err(map_account_snapshot_write_error)?;
                if !updated {
                    return Err(ApiError::NotFound);
                }
                overwritten_count += 1;
            }
            SnapshotImportPreviewAction::SkipExisting
            | SnapshotImportPreviewAction::SkipUnchanged
            | SnapshotImportPreviewAction::SkipDuplicate
            | SnapshotImportPreviewAction::SkipBlankAmount
            | SnapshotImportPreviewAction::Invalid => {}
        }
    }

    tx.commit().await.map_err(|_| ApiError::Db)?;

    Ok(SnapshotImportCommitDto {
        created_count,
        overwritten_count,
        skipped_count: plan.preview.summary.skip_count,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_delete_preview(
    state: State<'_, AppState>,
    institution_id: i64,
) -> Result<InstitutionDeletePreviewDto, ApiError> {
    let pool = &state.pool;
    let Some(institution) = db::institution_get(pool, institution_id)
        .await
        .map_err(|_| ApiError::Db)?
    else {
        return Err(ApiError::NotFound);
    };

    let accounts = db::institution_accounts_delete_preview(pool, institution_id)
        .await
        .map_err(|_| ApiError::Db)?
        .into_iter()
        .map(|row| InstitutionDeletePreviewAccountDto {
            id: row.id,
            name: row.name,
            snapshot_count: u32::try_from(row.snapshot_count)
                .expect("snapshot count should fit in u32"),
        })
        .collect::<Vec<_>>();

    let total_snapshots = accounts.iter().map(|account| account.snapshot_count).sum();

    Ok(InstitutionDeletePreviewDto {
        institution: InstitutionDto {
            id: institution.id,
            name: institution.name,
        },
        accounts,
        total_snapshots,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn institutions_delete(
    state: State<'_, AppState>,
    institution_id: i64,
) -> Result<(), ApiError> {
    let pool = &state.pool;
    let deleted = db::institution_delete(pool, institution_id)
        .await
        .map_err(|_| ApiError::Db)?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn account_balance_over_time(
    state: State<'_, AppState>,
    account_id: i64,
    period: BalanceOverTimePeriod,
) -> Result<Vec<BalancePointDto>, ApiError> {
    let today = Local::now().date_naive();
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
    let today = Local::now().date_naive();

    let accounts = db::accounts_list_full(pool)
        .await
        .map_err(|_| ApiError::Db)?;

    let mut total_balance_minor: i64 = 0;
    let mut active_accounts: u32 = 0;
    let mut active_institution_ids: HashSet<i64> = HashSet::new();
    let mut allocation: BTreeMap<AccountTypeName, i64> = BTreeMap::new();

    for a in &accounts {
        let account_type_name = a.type_name.parse().map_err(|_| ApiError::Db)?;
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
        total_accounts: u32::try_from(accounts.len()).expect("account count should fit in u32"),
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
    let today = Local::now().date_naive();
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
    closed_date: Option<NaiveDate>,
}

fn app_settings_dto_from_row(row: db::rows::AppSettingsRow) -> Result<AppSettingsDto, ApiError> {
    Ok(AppSettingsDto {
        analytics_enabled: row.analytics_enabled,
        default_display_currency_code: row
            .default_display_currency_code
            .parse()
            .map_err(|_| ApiError::Db)?,
        display_locale: row.display_locale.parse().map_err(|_| ApiError::Db)?,
        theme: row.theme.parse().map_err(|_| ApiError::Db)?,
    })
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

    let account_type_db = normalized.account_type.as_str();
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

    if let (Some(opened_date), Some(closed_date)) = (normalized.opened_date, normalized.closed_date)
    {
        if closed_date < opened_date {
            issues.push(validation_issue(
                "closed_date",
                "Closed date cannot be before opened date",
            ));
        }
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(ValidatedAccountUpsert {
        institution,
        name: normalized.name,
        type_id: type_id.expect("validated above"),
        currency_code: normalized.currency_code.as_str().to_owned(),
        normal_balance_sign: normalized.normal_balance_sign,
        opened_date: normalized.opened_date,
        closed_date: normalized.closed_date,
    })
}

async fn validate_account_snapshots_create(
    pool: &SqlitePool,
    account_id: i64,
    input: &AccountSnapshotsCreateInput,
) -> Result<(), ApiError> {
    let mut issues = validation_issues_from_garde_report(input.validate().err());
    let mut seen_dates = HashMap::<NaiveDate, usize>::new();
    let mut unique_dates = Vec::<NaiveDate>::new();
    let mut previous_date = None;

    for (index, snapshot) in input.snapshots.iter().enumerate() {
        if let Some(last_date) = previous_date {
            if snapshot.date <= last_date {
                issues.push(validation_issue(
                    &format!("snapshots.{index}.date"),
                    "Snapshots must be in ascending date order",
                ));
            }
        }
        previous_date = Some(snapshot.date);

        if let Some(existing_index) = seen_dates.insert(snapshot.date, index) {
            issues.push(validation_issue(
                &format!("snapshots.{existing_index}.date"),
                "Duplicate snapshot date",
            ));
            issues.push(validation_issue(
                &format!("snapshots.{index}.date"),
                "Duplicate snapshot date",
            ));
        } else {
            unique_dates.push(snapshot.date);
        }
    }

    if !unique_dates.is_empty() {
        let existing_by_date = db::snapshots_for_account_dates(pool, account_id, &unique_dates)
            .await
            .map_err(|_| ApiError::Db)?
            .into_iter()
            .map(|row| (row.balance_date, row.id))
            .collect::<HashMap<_, _>>();

        for (index, snapshot) in input.snapshots.iter().enumerate() {
            if existing_by_date.contains_key(&snapshot.date) && !snapshot.overwrite_existing {
                issues.push(validation_issue(
                    &format!("snapshots.{index}.overwrite_existing"),
                    "This date already exists. Confirm overwrite to continue",
                ));
            }
        }
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(())
}

async fn validate_account_snapshot_update(
    pool: &SqlitePool,
    account_id: i64,
    snapshot_id: i64,
    input: &AccountSnapshotUpdateInput,
) -> Result<(), ApiError> {
    let mut issues = validation_issues_from_garde_report(input.validate().err());

    let conflicting = db::snapshots_for_account_dates(pool, account_id, &[input.date])
        .await
        .map_err(|_| ApiError::Db)?
        .into_iter()
        .find(|snapshot| snapshot.id != snapshot_id);
    if conflicting.is_some() && !input.overwrite_existing {
        issues.push(validation_issue(
            "overwrite_existing",
            "This date already exists. Confirm overwrite to continue",
        ));
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(())
}

fn validate_account_snapshots_delete(
    input: &AccountSnapshotsDeleteInput,
) -> Result<Vec<i64>, ApiError> {
    let mut issues = validation_issues_from_garde_report(input.validate().err());
    let mut snapshot_ids = Vec::with_capacity(input.snapshot_ids.len());
    let mut seen = HashSet::new();

    for (index, snapshot_id) in input.snapshot_ids.iter().enumerate() {
        if *snapshot_id < 1 {
            issues.push(validation_issue(
                &format!("snapshot_ids.{index}"),
                "Snapshot id must be greater than 0",
            ));
            continue;
        }

        if seen.insert(*snapshot_id) {
            snapshot_ids.push(*snapshot_id);
        }
    }

    if !issues.is_empty() {
        return Err(ApiError::Validation(issues));
    }

    Ok(snapshot_ids)
}

/// Creates a validation issue whose UI message is also safe to send as telemetry.
///
/// Use this only when `message` does not include user-entered values, raw imported
/// data, or other sensitive details.
fn validation_issue(field: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        field: field.to_string(),
        message: message.to_string(),
        telemetry_message: Some(message.to_string()),
    }
}

/// Creates a validation issue with a separately controlled telemetry message.
///
/// Use this when the UI message is useful locally but may contain sensitive details.
/// Pass `None` to suppress the message from telemetry, or pass a redacted/generalized
/// alternative that is safe to send.
fn validation_issue_with_telemetry_message(
    field: &str,
    message: &str,
    telemetry_message: Option<&str>,
) -> ValidationIssue {
    ValidationIssue {
        field: field.to_string(),
        message: message.to_string(),
        telemetry_message: telemetry_message.map(str::to_string),
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

fn map_snapshot_import_validation(issues: Vec<SnapshotImportValidationIssue>) -> ApiError {
    ApiError::Validation(
        issues
            .into_iter()
            .map(|issue| {
                validation_issue_with_telemetry_message(
                    &issue.field,
                    &issue.message,
                    issue.telemetry_message.as_deref(),
                )
            })
            .collect(),
    )
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
        currency_code: input.currency_code,
        normal_balance_sign: input.normal_balance_sign,
        opened_date: input.opened_date,
        closed_date: input.closed_date,
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

fn map_account_snapshot_write_error(error: sqlx::Error) -> ApiError {
    if is_unique_constraint(
        &error,
        "account_balance_snapshots.account_id, account_balance_snapshots.balance_date",
    ) {
        return ApiError::Validation(vec![validation_issue(
            "date",
            "A snapshot already exists for this date",
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
    let today = Local::now().date_naive();
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
        let account_type_name = a.type_name.parse().map_err(|_| ApiError::Db)?;

        let institution = InstitutionDto {
            id: a.institution_id,
            name: a.institution_name,
        };
        let account_type = AccountTypeDto {
            id: a.type_id,
            name: account_type_name,
        };

        let latest_balance_minor = a.latest_balance_minor.unwrap_or(0);

        let date_map = snapshots_by_account.get(&a.id).unwrap_or(&empty_dates);
        let seed_before = initial_before.get(&a.id).copied();
        let series_180 = filled_values_for_period(date_map, seed_before, full_start, full_points);
        let monthly_change_minor = series_180.last().copied().flatten().unwrap_or(0)
            - series_180
                .get(series_180.len().saturating_sub(31))
                .copied()
                .flatten()
                .unwrap_or(0);

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
            currency_code: a.currency_code.parse().map_err(|_| ApiError::Db)?,
            normal_balance_sign: a.normal_balance_sign,
            opened_date: a.opened_date,
            closed_date: a.closed_date,
            first_snapshot_date: a.first_snapshot_date,
            latest_snapshot_date: a.latest_snapshot_date,
            latest_balance_minor,
            monthly_change_minor,
            activity_by_period,
        });
    }

    Ok(out)
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
        settings_get,
        settings_update,
        accounts_list,
        accounts_create,
        accounts_update,
        accounts_delete_preview,
        accounts_delete,
        institutions_list,
        institutions_create,
        institutions_update,
        institutions_delete_preview,
        institutions_delete,
        institutions_get,
        accounts_get,
        account_snapshots_list,
        account_snapshots_create,
        account_snapshot_update,
        account_snapshots_delete,
        account_snapshot_import_inspect,
        account_snapshot_import_preview,
        account_snapshot_import_commit,
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local, NaiveDate};
    use serde_json::json;
    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        SqlitePool,
    };
    use std::str::FromStr;

    use super::{
        account_snapshot_import_commit_with_today, build_account_dtos, ApiError,
        SnapshotImportOptionsInput, SnapshotImportSourceInput,
    };
    use crate::db;

    #[tokio::test]
    async fn account_monthly_change_uses_zero_when_no_balance_thirty_days_ago() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;
        let today = Local::now().date_naive();
        insert_snapshot_on(&pool, account_id, today - Duration::days(10), 1000).await;

        let account = db::account_get_full(&pool, account_id)
            .await
            .unwrap()
            .unwrap();
        let dto = build_account_dtos(&pool, vec![account])
            .await
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(dto.monthly_change_minor, 1000);
    }

    #[tokio::test]
    async fn account_monthly_change_ignores_future_dated_snapshots() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;
        let today = Local::now().date_naive();
        insert_snapshot_on(&pool, account_id, today + Duration::days(1), 1000).await;

        let account = db::account_get_full(&pool, account_id)
            .await
            .unwrap()
            .unwrap();
        let dto = build_account_dtos(&pool, vec![account])
            .await
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(dto.monthly_change_minor, 0);
    }

    #[tokio::test]
    async fn account_snapshot_import_commit_returns_not_found_for_missing_account() {
        let pool = test_pool().await;

        let result = account_snapshot_import_commit_with_today(
            &pool,
            999,
            csv_input("date,balance\n2026-01-09,1.00\n"),
            import_options("overwrite", "include", "keep_last", true),
            date(2026, 1, 9),
        )
        .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    #[tokio::test]
    async fn account_snapshot_import_commit_blocks_invalid_preview_without_writes() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;

        let result = account_snapshot_import_commit_with_today(
            &pool,
            account_id,
            csv_input("date,balance\nnot-a-date,1.00\n2026-01-09,2.00\n"),
            import_options("overwrite", "include", "keep_last", true),
            date(2026, 1, 9),
        )
        .await;

        assert_validation_error(
            result,
            "import",
            "Fix invalid rows before importing snapshots",
        );
        assert_eq!(
            snapshot_balances(&pool, account_id).await,
            Vec::<(NaiveDate, i64)>::new()
        );
    }

    #[tokio::test]
    async fn account_snapshot_import_commit_requires_overwrite_confirmation() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;
        insert_snapshot(&pool, account_id, 2026, 1, 9, 100).await;

        let result = account_snapshot_import_commit_with_today(
            &pool,
            account_id,
            csv_input("date,balance\n2026-01-09,2.00\n"),
            import_options("overwrite", "include", "keep_last", false),
            date(2026, 1, 9),
        )
        .await;

        assert_validation_error(result, "import", "Confirm overwrite to continue");
        assert_eq!(
            snapshot_balances(&pool, account_id).await,
            vec![(date(2026, 1, 9), 100)]
        );
    }

    #[tokio::test]
    async fn account_snapshot_import_commit_persists_creates_overwrites_and_skips() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;
        insert_snapshot(&pool, account_id, 2026, 1, 8, 100).await;
        insert_snapshot(&pool, account_id, 2026, 1, 10, 500).await;

        let result = account_snapshot_import_commit_with_today(
            &pool,
            account_id,
            csv_input(
                "date,balance\n2026-01-09,1.00\n2026-01-10,6.00\n2026-01-11,7.00\n2026-01-11,8.00\n",
            ),
            import_options("overwrite", "exclude", "keep_last", true),
            date(2026, 1, 9),
        )
        .await
        .unwrap();

        assert_eq!(result.created_count, 1);
        assert_eq!(result.overwritten_count, 1);
        assert_eq!(result.skipped_count, 2);
        assert_eq!(
            snapshot_balances(&pool, account_id).await,
            vec![
                (date(2026, 1, 8), 100),
                (date(2026, 1, 10), 600),
                (date(2026, 1, 11), 800),
            ]
        );
    }

    #[tokio::test]
    async fn account_snapshot_import_commit_does_not_write_skip_existing_rows() {
        let pool = test_pool().await;
        let account_id = create_account(&pool).await;
        insert_snapshot(&pool, account_id, 2026, 1, 9, 100).await;

        let result = account_snapshot_import_commit_with_today(
            &pool,
            account_id,
            csv_input("date,balance\n2026-01-09,2.00\n2026-01-10,2.00\n"),
            import_options("skip", "include", "keep_last", false),
            date(2026, 1, 9),
        )
        .await
        .unwrap();

        assert_eq!(result.created_count, 1);
        assert_eq!(result.overwritten_count, 0);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(
            snapshot_balances(&pool, account_id).await,
            vec![(date(2026, 1, 9), 100), (date(2026, 1, 10), 200)]
        );
    }

    async fn test_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();

        sqlx::migrate!("./db/migrations").run(&pool).await.unwrap();
        pool
    }

    async fn create_account(pool: &SqlitePool) -> i64 {
        let institution_id = sqlx::query("INSERT INTO institutions (name) VALUES ('Bank')")
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid();
        let type_id =
            sqlx::query_scalar::<_, i64>("SELECT id FROM account_types WHERE name = 'current'")
                .fetch_one(pool)
                .await
                .unwrap();

        sqlx::query(
            r"
            INSERT INTO accounts (
                name,
                institution_id,
                type_id,
                currency_code,
                normal_balance_sign
            )
            VALUES ('Everyday', ?, ?, 'GBP', 1)
            ",
        )
        .bind(institution_id)
        .bind(type_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
    }

    async fn insert_snapshot(
        pool: &SqlitePool,
        account_id: i64,
        year: i32,
        month: u32,
        day: u32,
        balance_minor: i64,
    ) {
        insert_snapshot_on(pool, account_id, date(year, month, day), balance_minor).await;
    }

    async fn insert_snapshot_on(
        pool: &SqlitePool,
        account_id: i64,
        balance_date: NaiveDate,
        balance_minor: i64,
    ) {
        sqlx::query(
            r"
            INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor)
            VALUES (?, ?, ?)
            ",
        )
        .bind(account_id)
        .bind(balance_date)
        .bind(balance_minor)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn snapshot_balances(pool: &SqlitePool, account_id: i64) -> Vec<(NaiveDate, i64)> {
        db::snapshots_for_account(pool, account_id)
            .await
            .unwrap()
            .into_iter()
            .rev()
            .map(|snapshot| (snapshot.balance_date, snapshot.balance_minor))
            .collect()
    }

    fn csv_input(content: &str) -> SnapshotImportSourceInput {
        serde_json::from_value(json!({
            "kind": "csv",
            "file_name": "snapshots.csv",
            "content": content,
            "has_header_row": true,
        }))
        .unwrap()
    }

    fn import_options(
        existing_date_policy: &str,
        unchanged_value_policy: &str,
        duplicate_date_policy: &str,
        overwrite_existing_confirmed: bool,
    ) -> SnapshotImportOptionsInput {
        serde_json::from_value(json!({
            "source": {
                "kind": "csv",
                "date_column": "date",
                "amount_column": "balance",
                "date_format": "yyyy_mm_dd",
                "timestamp_date_policy": "date_as_written",
                "timestamp_missing_timezone_policy": "local",
                "timestamp_missing_timezone": "Europe/London",
                "balance_format": "thousands_comma_decimal_dot",
            },
            "existing_date_policy": existing_date_policy,
            "unchanged_value_policy": unchanged_value_policy,
            "duplicate_date_policy": duplicate_date_policy,
            "overwrite_existing_confirmed": overwrite_existing_confirmed,
        }))
        .unwrap()
    }

    fn assert_validation_error<T>(result: Result<T, ApiError>, field: &str, message: &str) {
        let Err(ApiError::Validation(issues)) = result else {
            panic!("expected validation error");
        };
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, field);
        assert_eq!(issues[0].message, message);
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }
}
