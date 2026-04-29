mod csv;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeMap, HashMap};

use crate::db::rows::AccountBalanceSnapshotRow;

pub use csv::{
    CsvSnapshotImportInspectionDto, CsvSnapshotImportOptionsInput, CsvSnapshotImportSourceInput,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SnapshotImportSourceInput {
    Csv(CsvSnapshotImportSourceInput),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SnapshotImportSourceOptionsInput {
    Csv(CsvSnapshotImportOptionsInput),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotImportExistingDatePolicy {
    Overwrite,
    Skip,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotImportUnchangedValuePolicy {
    Exclude,
    Include,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotImportDuplicateDatePolicy {
    KeepFirst,
    KeepLast,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportOptionsInput {
    pub source: SnapshotImportSourceOptionsInput,
    pub existing_date_policy: SnapshotImportExistingDatePolicy,
    pub unchanged_value_policy: SnapshotImportUnchangedValuePolicy,
    pub duplicate_date_policy: SnapshotImportDuplicateDatePolicy,
    pub overwrite_existing_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SnapshotImportInspectionDto {
    Csv(CsvSnapshotImportInspectionDto),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportPreviewSummaryDto {
    pub total_rows: u32,
    pub create_count: u32,
    pub overwrite_count: u32,
    pub skip_count: u32,
    pub invalid_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportExistingSnapshotDto {
    pub id: i64,
    pub date: NaiveDate,
    pub balance_minor: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotImportPreviewAction {
    Create,
    Overwrite,
    SkipExisting,
    SkipUnchanged,
    SkipDuplicate,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportPreviewRowDto {
    pub source_row_number: u32,
    pub raw_date: String,
    pub raw_amount: String,
    pub date: Option<NaiveDate>,
    pub balance_minor: Option<i64>,
    pub action: SnapshotImportPreviewAction,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub existing_snapshot: Option<SnapshotImportExistingSnapshotDto>,
    pub previous_balance_minor: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportPreviewDto {
    pub summary: SnapshotImportPreviewSummaryDto,
    pub rows: Vec<SnapshotImportPreviewRowDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SnapshotImportCommitDto {
    pub created_count: u32,
    pub overwritten_count: u32,
    pub skipped_count: u32,
}

#[derive(Debug, Clone)]
pub struct SnapshotImportValidationIssue {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SnapshotImportPlan {
    pub preview: SnapshotImportPreviewDto,
    pub writes: Vec<SnapshotImportWrite>,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapshotImportPlanningContext {
    pub account_opened_date: Option<NaiveDate>,
    pub account_closed_date: Option<NaiveDate>,
    pub today: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct SnapshotImportWrite {
    pub action: SnapshotImportPreviewAction,
    pub date: NaiveDate,
    pub balance_minor: i64,
    pub existing_snapshot_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub(crate) struct SnapshotImportCandidate {
    pub source_row_number: u32,
    pub raw_date: String,
    pub raw_amount: String,
    pub date: Option<NaiveDate>,
    pub balance_minor: Option<i64>,
    pub issues: Vec<String>,
    pub skip_duplicate: bool,
}

pub fn inspect_source(
    input: &SnapshotImportSourceInput,
) -> Result<SnapshotImportInspectionDto, Vec<SnapshotImportValidationIssue>> {
    match input {
        SnapshotImportSourceInput::Csv(input) => {
            csv::inspect(input).map(SnapshotImportInspectionDto::Csv)
        }
    }
}

pub fn plan_import(
    input: &SnapshotImportSourceInput,
    options: &SnapshotImportOptionsInput,
    existing_snapshots: &[AccountBalanceSnapshotRow],
    context: SnapshotImportPlanningContext,
) -> Result<SnapshotImportPlan, Vec<SnapshotImportValidationIssue>> {
    let mut candidates = match (input, &options.source) {
        (SnapshotImportSourceInput::Csv(input), SnapshotImportSourceOptionsInput::Csv(options)) => {
            csv::candidates(input, options)?
        }
    };

    resolve_duplicate_dates(&mut candidates, options.duplicate_date_policy);
    Ok(build_plan(candidates, options, existing_snapshots, context))
}

fn resolve_duplicate_dates(
    rows: &mut [SnapshotImportCandidate],
    policy: SnapshotImportDuplicateDatePolicy,
) {
    let mut row_indexes_by_date = HashMap::<NaiveDate, Vec<usize>>::new();
    for (index, date) in rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.date.map(|date| (index, date)))
    {
        row_indexes_by_date.entry(date).or_default().push(index);
    }

    for row_indexes in row_indexes_by_date
        .values()
        .filter(|indexes| indexes.len() > 1)
    {
        match policy {
            SnapshotImportDuplicateDatePolicy::Error => {
                for index in row_indexes {
                    rows[*index]
                        .issues
                        .push("Duplicate date in selected import file".to_string());
                }
            }
            SnapshotImportDuplicateDatePolicy::KeepFirst => {
                for index in row_indexes.iter().skip(1) {
                    rows[*index].skip_duplicate = true;
                }
            }
            SnapshotImportDuplicateDatePolicy::KeepLast => {
                for index in row_indexes.iter().rev().skip(1) {
                    rows[*index].skip_duplicate = true;
                }
            }
        }
    }
}

fn build_plan(
    candidates: Vec<SnapshotImportCandidate>,
    options: &SnapshotImportOptionsInput,
    existing_snapshots: &[AccountBalanceSnapshotRow],
    context: SnapshotImportPlanningContext,
) -> SnapshotImportPlan {
    let existing_by_date = existing_snapshots
        .iter()
        .map(|snapshot| (snapshot.balance_date, snapshot))
        .collect::<HashMap<_, _>>();
    let mut timeline = existing_snapshots
        .iter()
        .map(|snapshot| (snapshot.balance_date, snapshot.balance_minor))
        .collect::<BTreeMap<_, _>>();
    let mut planned_by_row = HashMap::<u32, SnapshotImportPreviewRowDto>::new();
    let mut writes = Vec::<SnapshotImportWrite>::new();
    let mut chronological = candidates.clone();
    chronological.sort_by_key(|row| (row.date, row.source_row_number));

    for row in chronological {
        let Some(date) = row.date else {
            planned_by_row.insert(row.source_row_number, invalid_preview_row(row, None, None));
            continue;
        };

        let previous_balance_minor = timeline.range(..date).next_back().map(|(_, value)| *value);
        let existing_snapshot = existing_by_date
            .get(&date)
            .map(|snapshot| existing_snapshot_dto(snapshot));

        if row.skip_duplicate {
            planned_by_row.insert(
                row.source_row_number,
                preview_row(
                    row,
                    SnapshotImportPreviewAction::SkipDuplicate,
                    existing_snapshot,
                    previous_balance_minor,
                ),
            );
            continue;
        }

        let Some(balance_minor) = row.balance_minor else {
            planned_by_row.insert(
                row.source_row_number,
                invalid_preview_row(row, existing_snapshot, previous_balance_minor),
            );
            continue;
        };

        if !row.issues.is_empty() {
            planned_by_row.insert(
                row.source_row_number,
                invalid_preview_row(row, existing_snapshot, previous_balance_minor),
            );
            continue;
        }

        if existing_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.balance_minor == balance_minor)
        {
            planned_by_row.insert(
                row.source_row_number,
                preview_row(
                    row,
                    SnapshotImportPreviewAction::SkipUnchanged,
                    existing_snapshot,
                    previous_balance_minor,
                ),
            );
            continue;
        }

        if existing_snapshot.is_some()
            && options.existing_date_policy == SnapshotImportExistingDatePolicy::Error
        {
            let mut row = row;
            row.issues
                .push("Snapshot already exists for this date".to_string());
            planned_by_row.insert(
                row.source_row_number,
                invalid_preview_row(row, existing_snapshot, previous_balance_minor),
            );
            continue;
        }

        if existing_snapshot.is_some()
            && options.existing_date_policy == SnapshotImportExistingDatePolicy::Skip
        {
            planned_by_row.insert(
                row.source_row_number,
                preview_row(
                    row,
                    SnapshotImportPreviewAction::SkipExisting,
                    existing_snapshot,
                    previous_balance_minor,
                ),
            );
            continue;
        }

        if existing_snapshot.is_none()
            && previous_balance_minor == Some(balance_minor)
            && options.unchanged_value_policy == SnapshotImportUnchangedValuePolicy::Exclude
        {
            planned_by_row.insert(
                row.source_row_number,
                preview_row(
                    row,
                    SnapshotImportPreviewAction::SkipUnchanged,
                    existing_snapshot,
                    previous_balance_minor,
                ),
            );
            continue;
        }

        let action = if existing_snapshot.is_some() {
            SnapshotImportPreviewAction::Overwrite
        } else {
            SnapshotImportPreviewAction::Create
        };
        timeline.insert(date, balance_minor);
        writes.push(SnapshotImportWrite {
            action,
            date,
            balance_minor,
            existing_snapshot_id: existing_snapshot.as_ref().map(|snapshot| snapshot.id),
        });
        planned_by_row.insert(
            row.source_row_number,
            preview_row(row, action, existing_snapshot, previous_balance_minor),
        );
    }

    let write_dates = writes.iter().map(|write| write.date).collect::<Vec<_>>();
    let rows = candidates
        .into_iter()
        .filter_map(|row| planned_by_row.remove(&row.source_row_number))
        .map(|mut row| {
            row.warnings = warnings_for_row(&row, context, &write_dates, existing_snapshots);
            row
        })
        .collect::<Vec<_>>();
    SnapshotImportPlan {
        preview: SnapshotImportPreviewDto {
            summary: summary_for_rows(&rows),
            rows,
        },
        writes,
    }
}

fn invalid_preview_row(
    row: SnapshotImportCandidate,
    existing_snapshot: Option<SnapshotImportExistingSnapshotDto>,
    previous_balance_minor: Option<i64>,
) -> SnapshotImportPreviewRowDto {
    preview_row(
        row,
        SnapshotImportPreviewAction::Invalid,
        existing_snapshot,
        previous_balance_minor,
    )
}

fn preview_row(
    row: SnapshotImportCandidate,
    action: SnapshotImportPreviewAction,
    existing_snapshot: Option<SnapshotImportExistingSnapshotDto>,
    previous_balance_minor: Option<i64>,
) -> SnapshotImportPreviewRowDto {
    SnapshotImportPreviewRowDto {
        source_row_number: row.source_row_number,
        raw_date: row.raw_date,
        raw_amount: row.raw_amount,
        date: row.date,
        balance_minor: row.balance_minor,
        action,
        issues: row.issues,
        warnings: Vec::new(),
        existing_snapshot,
        previous_balance_minor,
    }
}

fn warnings_for_row(
    row: &SnapshotImportPreviewRowDto,
    context: SnapshotImportPlanningContext,
    write_dates: &[NaiveDate],
    existing_snapshots: &[AccountBalanceSnapshotRow],
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(date) = row.date else {
        return warnings;
    };

    if date > context.today
        && matches!(
            row.action,
            SnapshotImportPreviewAction::Create | SnapshotImportPreviewAction::Overwrite
        )
    {
        let has_later_snapshot = write_dates
            .iter()
            .any(|candidate_date| *candidate_date > date)
            || existing_snapshots
                .iter()
                .any(|snapshot| snapshot.balance_date > date);

        if has_later_snapshot {
            warnings.push(
                "Future-dated snapshot. Balance-over-time charts only show data through today."
                    .to_string(),
            );
        } else {
            warnings.push(
                "Future-dated snapshot. This snapshot will count as the latest balance, but balance-over-time charts only show data through today."
                    .to_string(),
            );
        }
    }

    if let Some(opened_date) = context
        .account_opened_date
        .filter(|opened_date| date < *opened_date)
    {
        warnings.push(format!(
            "Snapshot is before the account opened date of {opened_date}."
        ));
    }

    if let Some(closed_date) = context
        .account_closed_date
        .filter(|closed_date| date > *closed_date)
    {
        warnings.push(format!(
            "Snapshot is after the account closed date of {closed_date}."
        ));
    }

    warnings
}

fn summary_for_rows(rows: &[SnapshotImportPreviewRowDto]) -> SnapshotImportPreviewSummaryDto {
    SnapshotImportPreviewSummaryDto {
        total_rows: u32::try_from(rows.len()).expect("row count should fit in u32"),
        create_count: count_rows(rows, SnapshotImportPreviewAction::Create),
        overwrite_count: count_rows(rows, SnapshotImportPreviewAction::Overwrite),
        skip_count: u32::try_from(
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.action,
                        SnapshotImportPreviewAction::SkipExisting
                            | SnapshotImportPreviewAction::SkipUnchanged
                            | SnapshotImportPreviewAction::SkipDuplicate
                    )
                })
                .count(),
        )
        .expect("row count should fit in u32"),
        invalid_count: count_rows(rows, SnapshotImportPreviewAction::Invalid),
    }
}

fn count_rows(rows: &[SnapshotImportPreviewRowDto], action: SnapshotImportPreviewAction) -> u32 {
    u32::try_from(rows.iter().filter(|row| row.action == action).count())
        .expect("row count should fit in u32")
}

fn existing_snapshot_dto(
    snapshot: &AccountBalanceSnapshotRow,
) -> SnapshotImportExistingSnapshotDto {
    SnapshotImportExistingSnapshotDto {
        id: snapshot.id,
        date: snapshot.balance_date,
        balance_minor: snapshot.balance_minor,
        created_at: snapshot.created_at,
    }
}

pub(crate) fn issue(field: &str, message: &str) -> SnapshotImportValidationIssue {
    SnapshotImportValidationIssue {
        field: field.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    use super::csv::{
        CsvSnapshotImportBalanceFormat, CsvSnapshotImportDateFormat,
        CsvSnapshotImportMissingTimezonePolicy, CsvSnapshotImportTimestampDatePolicy,
    };
    use super::{
        plan_import, resolve_duplicate_dates, CsvSnapshotImportOptionsInput,
        CsvSnapshotImportSourceInput, SnapshotImportCandidate, SnapshotImportDuplicateDatePolicy,
        SnapshotImportExistingDatePolicy, SnapshotImportOptionsInput,
        SnapshotImportPlanningContext, SnapshotImportPreviewAction, SnapshotImportSourceInput,
        SnapshotImportSourceOptionsInput, SnapshotImportUnchangedValuePolicy,
    };
    use crate::db::rows::AccountBalanceSnapshotRow;

    #[test]
    fn resolve_duplicate_dates_can_keep_first_row_for_date() {
        let mut rows = duplicate_rows();

        resolve_duplicate_dates(&mut rows, SnapshotImportDuplicateDatePolicy::KeepFirst);

        assert!(!rows[0].skip_duplicate);
        assert!(rows[1].skip_duplicate);
        assert!(!rows[2].skip_duplicate);
        assert!(rows.iter().all(|row| row.issues.is_empty()));
    }

    #[test]
    fn resolve_duplicate_dates_can_keep_last_row_for_date() {
        let mut rows = duplicate_rows();

        resolve_duplicate_dates(&mut rows, SnapshotImportDuplicateDatePolicy::KeepLast);

        assert!(rows[0].skip_duplicate);
        assert!(!rows[1].skip_duplicate);
        assert!(!rows[2].skip_duplicate);
        assert!(rows.iter().all(|row| row.issues.is_empty()));
    }

    #[test]
    fn resolve_duplicate_dates_can_mark_duplicates_invalid() {
        let mut rows = duplicate_rows();

        resolve_duplicate_dates(&mut rows, SnapshotImportDuplicateDatePolicy::Error);

        assert!(rows[0]
            .issues
            .contains(&"Duplicate date in selected import file".to_string()));
        assert!(rows[1]
            .issues
            .contains(&"Duplicate date in selected import file".to_string()));
        assert!(rows[2].issues.is_empty());
        assert!(rows.iter().all(|row| !row.skip_duplicate));
    }

    #[test]
    fn plan_import_creates_chronologically_but_returns_preview_in_source_order() {
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-10,2.00\n2026-01-09,1.00\n2026-01-11,2.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &[],
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![
                SnapshotImportPreviewAction::Create,
                SnapshotImportPreviewAction::Create,
                SnapshotImportPreviewAction::SkipUnchanged,
            ]
        );
        assert_eq!(plan.preview.rows[0].date, date(2026, 1, 10));
        assert_eq!(plan.preview.rows[0].previous_balance_minor, Some(100));
        assert_eq!(plan.preview.rows[1].date, date(2026, 1, 9));
        assert_eq!(plan.preview.rows[1].previous_balance_minor, None);
        assert_eq!(
            plan.writes
                .iter()
                .map(|write| write.date)
                .collect::<Vec<_>>(),
            vec![date(2026, 1, 9).unwrap(), date(2026, 1, 10).unwrap()]
        );
        assert_eq!(plan.preview.summary.total_rows, 3);
        assert_eq!(plan.preview.summary.create_count, 2);
        assert_eq!(plan.preview.summary.skip_count, 1);
    }

    #[test]
    fn plan_import_overwrites_existing_date_when_policy_allows_it() {
        let existing = vec![snapshot(7, 2026, 1, 9, 100)];
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,2.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![SnapshotImportPreviewAction::Overwrite]
        );
        assert_eq!(plan.preview.summary.overwrite_count, 1);
        assert_eq!(plan.writes.len(), 1);
        assert_eq!(plan.writes[0].existing_snapshot_id, Some(7));
        assert_eq!(plan.writes[0].balance_minor, 200);
        assert_eq!(
            plan.preview.rows[0]
                .existing_snapshot
                .as_ref()
                .unwrap()
                .balance_minor,
            100
        );
    }

    #[test]
    fn plan_import_skips_existing_date_when_policy_says_skip() {
        let existing = vec![snapshot(7, 2026, 1, 9, 100)];
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,2.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Skip,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![SnapshotImportPreviewAction::SkipExisting]
        );
        assert_eq!(plan.preview.summary.skip_count, 1);
        assert!(plan.writes.is_empty());
    }

    #[test]
    fn plan_import_marks_existing_date_invalid_when_policy_says_error() {
        let existing = vec![snapshot(7, 2026, 1, 9, 100)];
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,2.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Error,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![SnapshotImportPreviewAction::Invalid]
        );
        assert_eq!(
            plan.preview.rows[0].issues,
            vec!["Snapshot already exists for this date"]
        );
        assert_eq!(plan.preview.summary.invalid_count, 1);
        assert!(plan.writes.is_empty());
    }

    #[test]
    fn plan_import_skips_same_existing_balance_before_existing_date_policy() {
        let existing = vec![snapshot(7, 2026, 1, 9, 100)];
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,1.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Error,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![SnapshotImportPreviewAction::SkipUnchanged]
        );
        assert!(plan.preview.rows[0].issues.is_empty());
        assert!(plan.writes.is_empty());
    }

    #[test]
    fn plan_import_applies_unchanged_value_policy_against_previous_effective_balance() {
        let existing = vec![snapshot(1, 2026, 1, 8, 100)];
        let excluded = plan_import(
            &csv_input("date,balance\n2026-01-09,1.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();
        let included = plan_import(
            &csv_input("date,balance\n2026-01-09,1.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &existing,
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&excluded.preview.rows),
            vec![SnapshotImportPreviewAction::SkipUnchanged]
        );
        assert!(excluded.writes.is_empty());
        assert_eq!(
            actions(&included.preview.rows),
            vec![SnapshotImportPreviewAction::Create]
        );
        assert_eq!(included.writes.len(), 1);
    }

    #[test]
    fn plan_import_uses_earlier_imported_rows_for_later_unchanged_detection() {
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-10,5.00\n2026-01-09,5.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Exclude,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &[],
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![
                SnapshotImportPreviewAction::SkipUnchanged,
                SnapshotImportPreviewAction::Create,
            ]
        );
        assert_eq!(plan.preview.rows[0].previous_balance_minor, Some(500));
        assert_eq!(plan.writes.len(), 1);
        assert_eq!(plan.writes[0].date, date(2026, 1, 9).unwrap());
    }

    #[test]
    fn plan_import_applies_duplicate_keep_last_policy_through_preview_and_writes() {
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,1.00\n2026-01-10,2.00\n2026-01-09,3.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &[],
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![
                SnapshotImportPreviewAction::SkipDuplicate,
                SnapshotImportPreviewAction::Create,
                SnapshotImportPreviewAction::Create,
            ]
        );
        assert_eq!(plan.preview.summary.create_count, 2);
        assert_eq!(plan.preview.summary.skip_count, 1);
        assert_eq!(
            plan.writes
                .iter()
                .map(|write| write.balance_minor)
                .collect::<Vec<_>>(),
            vec![300, 200]
        );
    }

    #[test]
    fn plan_import_marks_all_duplicate_dates_invalid_when_policy_errors() {
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-09,1.00\n2026-01-09,2.00\n2026-01-10,3.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::Error,
            ),
            &[],
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![
                SnapshotImportPreviewAction::Invalid,
                SnapshotImportPreviewAction::Invalid,
                SnapshotImportPreviewAction::Create,
            ]
        );
        assert!(plan.preview.rows[0]
            .issues
            .contains(&"Duplicate date in selected import file".to_string()));
        assert!(plan.preview.rows[1]
            .issues
            .contains(&"Duplicate date in selected import file".to_string()));
        assert_eq!(plan.preview.summary.invalid_count, 2);
        assert_eq!(plan.writes.len(), 1);
    }

    #[test]
    fn plan_import_preserves_invalid_rows_without_writes() {
        let plan = plan_import(
            &csv_input("date,balance\nnot-a-date,1.00\n2026-01-09,nope\n2026-01-10,2.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &[],
            context(),
        )
        .unwrap();

        assert_eq!(
            actions(&plan.preview.rows),
            vec![
                SnapshotImportPreviewAction::Invalid,
                SnapshotImportPreviewAction::Invalid,
                SnapshotImportPreviewAction::Create,
            ]
        );
        assert_eq!(
            plan.preview.rows[0].issues,
            vec!["Date does not match the selected format"]
        );
        assert_eq!(
            plan.preview.rows[1].issues,
            vec!["Amount is not a valid currency value"]
        );
        assert_eq!(plan.preview.summary.invalid_count, 2);
        assert_eq!(plan.writes.len(), 1);
    }

    #[test]
    fn plan_import_adds_future_and_account_date_warnings() {
        let context = SnapshotImportPlanningContext {
            account_opened_date: date(2026, 1, 9),
            account_closed_date: date(2026, 1, 10),
            today: date(2026, 1, 8).unwrap(),
        };
        let plan = plan_import(
            &csv_input("date,balance\n2026-01-07,1.00\n2026-01-09,2.00\n2026-01-11,3.00\n"),
            &options(
                SnapshotImportExistingDatePolicy::Overwrite,
                SnapshotImportUnchangedValuePolicy::Include,
                SnapshotImportDuplicateDatePolicy::KeepLast,
            ),
            &[],
            context,
        )
        .unwrap();

        assert_eq!(
            plan.preview.rows[0].warnings,
            vec!["Snapshot is before the account opened date of 2026-01-09."]
        );
        assert_eq!(
            plan.preview.rows[1].warnings,
            vec!["Future-dated snapshot. Balance-over-time charts only show data through today."]
        );
        assert_eq!(
            plan.preview.rows[2].warnings,
            vec![
                "Future-dated snapshot. This snapshot will count as the latest balance, but balance-over-time charts only show data through today.",
                "Snapshot is after the account closed date of 2026-01-10.",
            ]
        );
    }

    fn duplicate_rows() -> Vec<SnapshotImportCandidate> {
        vec![
            candidate(1, 2026, 1, 9),
            candidate(2, 2026, 1, 9),
            candidate(3, 2026, 1, 10),
        ]
    }

    fn candidate(
        source_row_number: u32,
        year: i32,
        month: u32,
        day: u32,
    ) -> SnapshotImportCandidate {
        SnapshotImportCandidate {
            source_row_number,
            raw_date: format!("{year:04}-{month:02}-{day:02}"),
            raw_amount: "1.00".to_string(),
            date: NaiveDate::from_ymd_opt(year, month, day),
            balance_minor: Some(100),
            issues: Vec::new(),
            skip_duplicate: false,
        }
    }

    fn csv_input(content: &str) -> SnapshotImportSourceInput {
        SnapshotImportSourceInput::Csv(CsvSnapshotImportSourceInput {
            file_name: "snapshots.csv".to_string(),
            content: content.to_string(),
            has_header_row: true,
        })
    }

    fn options(
        existing_date_policy: SnapshotImportExistingDatePolicy,
        unchanged_value_policy: SnapshotImportUnchangedValuePolicy,
        duplicate_date_policy: SnapshotImportDuplicateDatePolicy,
    ) -> SnapshotImportOptionsInput {
        SnapshotImportOptionsInput {
            source: SnapshotImportSourceOptionsInput::Csv(CsvSnapshotImportOptionsInput {
                date_column: "date".to_string(),
                amount_column: "balance".to_string(),
                date_format: CsvSnapshotImportDateFormat::YyyyMmDd,
                timestamp_date_policy: CsvSnapshotImportTimestampDatePolicy::DateAsWritten,
                timestamp_missing_timezone_policy: CsvSnapshotImportMissingTimezonePolicy::Local,
                timestamp_missing_timezone: "Europe/London".to_string(),
                balance_format: CsvSnapshotImportBalanceFormat::ThousandsCommaDecimalDot,
            }),
            existing_date_policy,
            unchanged_value_policy,
            duplicate_date_policy,
            overwrite_existing_confirmed: false,
        }
    }

    fn context() -> SnapshotImportPlanningContext {
        SnapshotImportPlanningContext {
            account_opened_date: None,
            account_closed_date: None,
            today: date(2026, 1, 8).unwrap(),
        }
    }

    fn snapshot(
        id: i64,
        year: i32,
        month: u32,
        day: u32,
        balance_minor: i64,
    ) -> AccountBalanceSnapshotRow {
        AccountBalanceSnapshotRow {
            id,
            account_id: 1,
            balance_date: date(year, month, day).unwrap(),
            balance_minor,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    fn actions(rows: &[super::SnapshotImportPreviewRowDto]) -> Vec<SnapshotImportPreviewAction> {
        rows.iter().map(|row| row.action).collect()
    }

    fn date(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(year, month, day)
    }
}
