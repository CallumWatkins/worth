use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

use super::{
    issue, SnapshotImportCandidate, SnapshotImportDuplicateDatePolicy,
    SnapshotImportValidationIssue,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportSourceInput {
    pub file_name: String,
    pub content: String,
    pub has_header_row: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportOptionsInput {
    pub date_column: String,
    pub amount_column: String,
    pub date_format: CsvSnapshotImportDateFormat,
    pub timestamp_date_policy: CsvSnapshotImportTimestampDatePolicy,
    pub timestamp_missing_timezone_policy: CsvSnapshotImportMissingTimezonePolicy,
    pub timestamp_missing_timezone: String,
    pub balance_format: CsvSnapshotImportBalanceFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvSnapshotImportDateFormat {
    YyyyMmDd,
    DdMmYyyySlash,
    DdMmYySlash,
    MmDdYyyySlash,
    MmDdYySlash,
    DdMmYyyyDash,
    YyyyMmDdSlash,
    #[serde(rename = "iso_8601_date_time")]
    Iso8601DateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvSnapshotImportTimestampDatePolicy {
    DateAsWritten,
    ConvertToLocal,
    ConvertToUtc,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvSnapshotImportMissingTimezonePolicy {
    Local,
    Utc,
    Timezone,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvSnapshotImportBalanceFormat {
    ThousandsCommaDecimalDot,
    ThousandsDotDecimalComma,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportColumnDto {
    pub name: String,
    pub index: u32,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportSampleRowDto {
    pub source_row_number: u32,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportGuessesDto {
    pub date_column: Option<String>,
    pub amount_column: Option<String>,
    pub date_format: Option<CsvSnapshotImportDateFormat>,
    pub balance_format: Option<CsvSnapshotImportBalanceFormat>,
    pub timestamp_missing_timezone_policy: Option<CsvSnapshotImportMissingTimezonePolicy>,
    pub duplicate_date_policy: Option<SnapshotImportDuplicateDatePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CsvSnapshotImportInspectionDto {
    pub file_name: String,
    pub columns: Vec<CsvSnapshotImportColumnDto>,
    pub sample_rows: Vec<CsvSnapshotImportSampleRowDto>,
    pub guesses: CsvSnapshotImportGuessesDto,
    pub total_rows: u32,
}

#[derive(Debug, Clone)]
struct CsvData {
    file_name: String,
    columns: Vec<String>,
    rows: Vec<CsvRow>,
}

#[derive(Debug, Clone)]
struct CsvRow {
    source_row_number: u32,
    values: Vec<String>,
}

pub fn inspect(
    input: &CsvSnapshotImportSourceInput,
) -> Result<CsvSnapshotImportInspectionDto, Vec<SnapshotImportValidationIssue>> {
    let data = parse_csv(input)?;
    let columns = data
        .columns
        .iter()
        .enumerate()
        .map(|(index, name)| CsvSnapshotImportColumnDto {
            name: name.clone(),
            index: u32::try_from(index).expect("column index should fit in u32"),
            sample_values: sample_values_for_column(&data.rows, index),
        })
        .collect::<Vec<_>>();
    let date_column = guess_date_column(&data);
    let amount_column = guess_amount_column(&data);
    let date_format = date_column
        .as_ref()
        .and_then(|column| column_index(&data.columns, column))
        .and_then(|index| guess_date_format(&data.rows, index));
    let balance_format = guess_balance_format(&data, amount_column.as_ref());
    let timestamp_missing_timezone_policy =
        guess_timestamp_missing_timezone_policy(date_column.as_ref());
    let duplicate_date_policy =
        guess_duplicate_date_policy(&data, date_column.as_ref(), date_format);

    Ok(CsvSnapshotImportInspectionDto {
        file_name: data.file_name,
        columns,
        sample_rows: data
            .rows
            .iter()
            .take(5)
            .map(|row| CsvSnapshotImportSampleRowDto {
                source_row_number: row.source_row_number,
                values: row.values.clone(),
            })
            .collect(),
        guesses: CsvSnapshotImportGuessesDto {
            date_column,
            amount_column,
            date_format,
            balance_format,
            timestamp_missing_timezone_policy,
            duplicate_date_policy,
        },
        total_rows: u32::try_from(data.rows.len()).expect("row count should fit in u32"),
    })
}

pub fn candidates(
    input: &CsvSnapshotImportSourceInput,
    options: &CsvSnapshotImportOptionsInput,
) -> Result<Vec<SnapshotImportCandidate>, Vec<SnapshotImportValidationIssue>> {
    let data = parse_csv(input)?;
    let date_index = require_column(
        &data.columns,
        &options.date_column,
        "options.source.date_column",
    )?;
    let amount_index = require_column(
        &data.columns,
        &options.amount_column,
        "options.source.amount_column",
    )?;

    Ok(data
        .rows
        .iter()
        .map(|row| parse_candidate(row, date_index, amount_index, options))
        .collect())
}

fn parse_csv(
    input: &CsvSnapshotImportSourceInput,
) -> Result<CsvData, Vec<SnapshotImportValidationIssue>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(input.has_header_row)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(input.content.as_bytes());

    let header_columns = if input.has_header_row {
        let headers = reader.headers().map_err(|error| {
            vec![issue(
                "source",
                &format!("Could not read CSV headers: {error}"),
            )]
        })?;
        normalize_headers(headers.iter().collect())
    } else {
        Vec::new()
    };

    let mut rows = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let source_row_number = index + if input.has_header_row { 2 } else { 1 };
        let record = record.map_err(|error| {
            vec![issue(
                "source",
                &format!("Could not read CSV row {source_row_number}: {error}"),
            )]
        })?;
        let values = record
            .iter()
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
        if values.iter().all(|value| value.is_empty()) {
            continue;
        }
        rows.push(CsvRow {
            source_row_number: u32::try_from(source_row_number)
                .expect("row number should fit in u32"),
            values,
        });
    }

    let columns = if input.has_header_row {
        header_columns
    } else {
        let column_count = rows.iter().map(|row| row.values.len()).max().unwrap_or(0);
        (0..column_count)
            .map(|index| format!("Column {}", index + 1))
            .collect::<Vec<_>>()
    };

    if columns.is_empty() {
        return Err(vec![issue(
            "source",
            "CSV must include at least one column",
        )]);
    }

    let rows = rows
        .into_iter()
        .map(|row| CsvRow {
            source_row_number: row.source_row_number,
            values: (0..columns.len())
                .map(|column_index| row.values.get(column_index).cloned().unwrap_or_default())
                .collect(),
        })
        .collect::<Vec<_>>();

    Ok(CsvData {
        file_name: input.file_name.clone(),
        columns,
        rows,
    })
}

fn normalize_headers(headers: Vec<&str>) -> Vec<String> {
    let mut seen = HashMap::<String, usize>::new();
    headers
        .into_iter()
        .enumerate()
        .map(|(index, header)| {
            let base = if header.trim().is_empty() {
                format!("Column {}", index + 1)
            } else {
                header.trim().to_string()
            };
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            if *count == 1 {
                base
            } else {
                format!("{base} ({count})")
            }
        })
        .collect()
}

fn require_column(
    columns: &[String],
    column: &str,
    field: &str,
) -> Result<usize, Vec<SnapshotImportValidationIssue>> {
    column_index(columns, column).ok_or_else(|| vec![issue(field, "Select a valid column")])
}

fn column_index(columns: &[String], column: &str) -> Option<usize> {
    columns.iter().position(|candidate| candidate == column)
}

fn sample_values_for_column(rows: &[CsvRow], column_index: usize) -> Vec<String> {
    rows.iter()
        .filter_map(|row| row.values.get(column_index))
        .filter(|value| !value.is_empty())
        .take(5)
        .cloned()
        .collect()
}

fn guess_date_column(data: &CsvData) -> Option<String> {
    data.columns
        .iter()
        .enumerate()
        .max_by_key(|(index, column)| {
            let name = column.to_lowercase();
            let name_score = if name.contains("date") {
                100
            } else if name.contains("posted") || name.contains("as of") {
                60
            } else {
                0
            };
            name_score + guess_date_format(&data.rows, *index).map_or(0, |_| 20)
        })
        .and_then(|(index, column)| {
            if guess_date_format(&data.rows, index).is_some() {
                Some(column.clone())
            } else {
                None
            }
        })
}

fn guess_amount_column(data: &CsvData) -> Option<String> {
    data.columns
        .iter()
        .enumerate()
        .max_by_key(|(index, column)| {
            let name = column.to_lowercase();
            let name_score = if name.contains("balance") {
                100
            } else if name.contains("amount") || name.contains("value") {
                70
            } else {
                0
            };
            let parse_score = data
                .rows
                .iter()
                .take(20)
                .filter(|row| {
                    row.values
                        .get(*index)
                        .and_then(|value| parse_amount_minor_any_format(value))
                        .is_some()
                })
                .count()
                * 5;
            name_score + parse_score
        })
        .and_then(|(index, column)| {
            let has_amount = data.rows.iter().take(20).any(|row| {
                row.values
                    .get(index)
                    .and_then(|value| parse_amount_minor_any_format(value))
                    .is_some()
            });
            if has_amount {
                Some(column.clone())
            } else {
                None
            }
        })
}

fn guess_balance_format(
    data: &CsvData,
    amount_column: Option<&String>,
) -> Option<CsvSnapshotImportBalanceFormat> {
    let values = amount_column
        .and_then(|column| column_index(&data.columns, column))
        .map(|index| {
            data.rows
                .iter()
                .take(20)
                .filter_map(|row| row.values.get(index))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            data.rows
                .iter()
                .take(20)
                .flat_map(|row| row.values.iter())
                .collect::<Vec<_>>()
        });

    amount_formats()
        .into_iter()
        .max_by_key(|format| {
            (
                values
                    .iter()
                    .filter(|value| parse_amount_minor(value, *format).is_some())
                    .count(),
                matches!(
                    format,
                    CsvSnapshotImportBalanceFormat::ThousandsCommaDecimalDot
                ),
            )
        })
        .filter(|format| {
            values
                .iter()
                .any(|value| parse_amount_minor(value, *format).is_some())
        })
}

fn parse_amount_minor_any_format(raw: &str) -> Option<i64> {
    amount_formats()
        .into_iter()
        .find_map(|format| parse_amount_minor(raw, format))
}

fn amount_formats() -> Vec<CsvSnapshotImportBalanceFormat> {
    vec![
        CsvSnapshotImportBalanceFormat::ThousandsCommaDecimalDot,
        CsvSnapshotImportBalanceFormat::ThousandsDotDecimalComma,
    ]
}

fn guess_date_format(rows: &[CsvRow], column_index: usize) -> Option<CsvSnapshotImportDateFormat> {
    date_formats()
        .into_iter()
        .max_by_key(|format| {
            rows.iter()
                .take(20)
                .filter(|row| {
                    row.values
                        .get(column_index)
                        .and_then(|value| {
                            parse_date(
                                value,
                                *format,
                                CsvSnapshotImportTimestampDatePolicy::DateAsWritten,
                                CsvSnapshotImportMissingTimezonePolicy::Local,
                                "+00:00",
                            )
                        })
                        .is_some()
                })
                .count()
        })
        .filter(|format| {
            rows.iter().take(20).any(|row| {
                row.values
                    .get(column_index)
                    .and_then(|value| {
                        parse_date(
                            value,
                            *format,
                            CsvSnapshotImportTimestampDatePolicy::DateAsWritten,
                            CsvSnapshotImportMissingTimezonePolicy::Local,
                            "+00:00",
                        )
                    })
                    .is_some()
            })
        })
}

fn guess_timestamp_missing_timezone_policy(
    date_column: Option<&String>,
) -> Option<CsvSnapshotImportMissingTimezonePolicy> {
    let name = date_column?.to_lowercase();
    (name.contains("utc") || name.contains("gmt"))
        .then_some(CsvSnapshotImportMissingTimezonePolicy::Utc)
}

fn guess_duplicate_date_policy(
    data: &CsvData,
    date_column: Option<&String>,
    date_format: Option<CsvSnapshotImportDateFormat>,
) -> Option<SnapshotImportDuplicateDatePolicy> {
    let date_index = date_column.and_then(|column| column_index(&data.columns, column))?;
    let date_format = date_format?;
    let dates = data
        .rows
        .iter()
        .filter_map(|row| {
            row.values.get(date_index).and_then(|value| {
                parse_date(
                    value,
                    date_format,
                    CsvSnapshotImportTimestampDatePolicy::DateAsWritten,
                    CsvSnapshotImportMissingTimezonePolicy::Local,
                    "+00:00",
                )
            })
        })
        .collect::<Vec<_>>();

    if dates.is_empty() {
        return None;
    }

    let ascending_steps = dates.windows(2).filter(|pair| pair[0] < pair[1]).count();
    let descending_steps = dates.windows(2).filter(|pair| pair[0] > pair[1]).count();

    Some(if descending_steps > ascending_steps {
        SnapshotImportDuplicateDatePolicy::KeepFirst
    } else {
        SnapshotImportDuplicateDatePolicy::KeepLast
    })
}

fn parse_candidate(
    row: &CsvRow,
    date_index: usize,
    amount_index: usize,
    options: &CsvSnapshotImportOptionsInput,
) -> SnapshotImportCandidate {
    let raw_date = row.values.get(date_index).cloned().unwrap_or_default();
    let raw_amount = row.values.get(amount_index).cloned().unwrap_or_default();
    let date = parse_date(
        &raw_date,
        options.date_format,
        options.timestamp_date_policy,
        options.timestamp_missing_timezone_policy,
        &options.timestamp_missing_timezone,
    );
    let balance_minor = parse_amount_minor(&raw_amount, options.balance_format);
    let mut issues = Vec::new();

    if raw_date.trim().is_empty() {
        issues.push("Missing date".to_string());
    } else if date.is_none() {
        issues.push("Date does not match the selected format".to_string());
    }
    if raw_amount.trim().is_empty() {
        issues.push("Missing amount".to_string());
    } else if balance_minor.is_none() {
        issues.push("Amount is not a valid currency value".to_string());
    }

    SnapshotImportCandidate {
        source_row_number: row.source_row_number,
        raw_date,
        raw_amount,
        date,
        balance_minor,
        issues,
        skip_duplicate: false,
    }
}

fn parse_date(
    raw: &str,
    format: CsvSnapshotImportDateFormat,
    timestamp_date_policy: CsvSnapshotImportTimestampDatePolicy,
    timestamp_missing_timezone_policy: CsvSnapshotImportMissingTimezonePolicy,
    timestamp_missing_timezone: &str,
) -> Option<NaiveDate> {
    let pattern = match format {
        CsvSnapshotImportDateFormat::YyyyMmDd => "%Y-%m-%d",
        CsvSnapshotImportDateFormat::DdMmYyyySlash => "%d/%m/%Y",
        CsvSnapshotImportDateFormat::DdMmYySlash => "%d/%m/%y",
        CsvSnapshotImportDateFormat::MmDdYyyySlash => "%m/%d/%Y",
        CsvSnapshotImportDateFormat::MmDdYySlash => "%m/%d/%y",
        CsvSnapshotImportDateFormat::DdMmYyyyDash => "%d-%m-%Y",
        CsvSnapshotImportDateFormat::YyyyMmDdSlash => "%Y/%m/%d",
        CsvSnapshotImportDateFormat::Iso8601DateTime => {
            return parse_iso_8601_date_time_date(
                raw,
                timestamp_date_policy,
                timestamp_missing_timezone_policy,
                timestamp_missing_timezone,
            )
        }
    };
    NaiveDate::parse_from_str(raw.trim(), pattern).ok()
}

fn parse_iso_8601_date_time_date(
    raw: &str,
    timestamp_date_policy: CsvSnapshotImportTimestampDatePolicy,
    timestamp_missing_timezone_policy: CsvSnapshotImportMissingTimezonePolicy,
    timestamp_missing_timezone: &str,
) -> Option<NaiveDate> {
    let parsed = parse_iso_8601_date_time(raw.trim())?;

    if timestamp_date_policy == CsvSnapshotImportTimestampDatePolicy::DateAsWritten {
        return Some(match parsed {
            ParsedIso8601DateTime::WithOffset(timestamp) => timestamp.date_naive(),
            ParsedIso8601DateTime::WithoutOffset(date_time) => date_time.date(),
        });
    }

    let timestamp = match parsed {
        ParsedIso8601DateTime::WithOffset(timestamp) => timestamp,
        ParsedIso8601DateTime::WithoutOffset(date_time) => {
            match timestamp_missing_timezone_policy {
                CsvSnapshotImportMissingTimezonePolicy::Local => Local
                    .from_local_datetime(&date_time)
                    .earliest()?
                    .with_timezone(&Utc),
                CsvSnapshotImportMissingTimezonePolicy::Utc => Utc.from_utc_datetime(&date_time),
                CsvSnapshotImportMissingTimezonePolicy::Timezone => timestamp_missing_timezone
                    .parse::<Tz>()
                    .ok()?
                    .from_local_datetime(&date_time)
                    .earliest()?
                    .with_timezone(&Utc),
            }
            .fixed_offset()
        }
    };

    Some(match timestamp_date_policy {
        CsvSnapshotImportTimestampDatePolicy::DateAsWritten => unreachable!("handled above"),
        CsvSnapshotImportTimestampDatePolicy::ConvertToLocal => {
            timestamp.with_timezone(&Local).date_naive()
        }
        CsvSnapshotImportTimestampDatePolicy::ConvertToUtc => {
            timestamp.with_timezone(&Utc).date_naive()
        }
    })
}

enum ParsedIso8601DateTime {
    WithOffset(DateTime<FixedOffset>),
    WithoutOffset(NaiveDateTime),
}

fn parse_iso_8601_date_time(raw: &str) -> Option<ParsedIso8601DateTime> {
    let normalized = raw.replace(',', ".");
    let normalized = normalized
        .strip_suffix('Z')
        .or_else(|| normalized.strip_suffix('z'))
        .map_or(normalized.clone(), |value| format!("{value}+00:00"));

    DateTime::parse_from_rfc3339(&normalized)
        .ok()
        .or_else(|| parse_iso_8601_date_time_with_offset(&normalized))
        .map(ParsedIso8601DateTime::WithOffset)
        .or_else(|| {
            parse_iso_8601_date_time_without_offset(&normalized)
                .map(ParsedIso8601DateTime::WithoutOffset)
        })
}

fn parse_iso_8601_date_time_with_offset(raw: &str) -> Option<DateTime<FixedOffset>> {
    [
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%dT%H:%M%:z",
        "%Y-%m-%dT%H:%M%z",
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M%:z",
        "%Y-%m-%d %H:%M%z",
        "%Y%m%dT%H%M%S%.f%:z",
        "%Y%m%dT%H%M%S%.f%z",
        "%Y%m%dT%H%M%:z",
        "%Y%m%dT%H%M%z",
        "%Y%m%d %H%M%S%.f%:z",
        "%Y%m%d %H%M%S%.f%z",
        "%Y%m%d %H%M%:z",
        "%Y%m%d %H%M%z",
    ]
    .into_iter()
    .find_map(|pattern| DateTime::parse_from_str(raw, pattern).ok())
}

fn parse_iso_8601_date_time_without_offset(raw: &str) -> Option<NaiveDateTime> {
    [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M",
        "%Y%m%dT%H%M%S%.f",
        "%Y%m%dT%H%M",
        "%Y%m%d %H%M%S%.f",
        "%Y%m%d %H%M",
    ]
    .into_iter()
    .find_map(|pattern| NaiveDateTime::parse_from_str(raw, pattern).ok())
}

fn date_formats() -> Vec<CsvSnapshotImportDateFormat> {
    vec![
        CsvSnapshotImportDateFormat::YyyyMmDd,
        CsvSnapshotImportDateFormat::DdMmYyyySlash,
        CsvSnapshotImportDateFormat::DdMmYySlash,
        CsvSnapshotImportDateFormat::MmDdYyyySlash,
        CsvSnapshotImportDateFormat::MmDdYySlash,
        CsvSnapshotImportDateFormat::DdMmYyyyDash,
        CsvSnapshotImportDateFormat::YyyyMmDdSlash,
        CsvSnapshotImportDateFormat::Iso8601DateTime,
    ]
}

fn parse_amount_minor(raw: &str, format: CsvSnapshotImportBalanceFormat) -> Option<i64> {
    let (amount, negative) = normalize_amount_text(raw)?;
    let (thousands_separator, decimal_separator) = match format {
        CsvSnapshotImportBalanceFormat::ThousandsCommaDecimalDot => (',', '.'),
        CsvSnapshotImportBalanceFormat::ThousandsDotDecimalComma => ('.', ','),
    };
    let mut parts = amount.split(decimal_separator);
    let integer_part = parts.next()?;
    let decimal_part = parts.next();
    if parts.next().is_some() || integer_part.is_empty() {
        return None;
    }

    if !valid_integer_part(integer_part, thousands_separator) {
        return None;
    }

    let cents = match decimal_part {
        Some(part)
            if part.len() <= 2
                && !part.is_empty()
                && part.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            part.parse::<i64>().ok()? * if part.len() == 1 { 10 } else { 1 }
        }
        Some(_) => return None,
        None => 0,
    };
    let units = integer_part
        .chars()
        .filter(|ch| *ch != thousands_separator)
        .collect::<String>()
        .parse::<i64>()
        .ok()?;
    let amount_minor = units.checked_mul(100)?.checked_add(cents)?;

    Some(if negative {
        -amount_minor
    } else {
        amount_minor
    })
}

fn normalize_amount_text(raw: &str) -> Option<(&str, bool)> {
    let mut value = raw.trim();
    if value.is_empty() {
        return None;
    }

    let negative_parentheses = value.starts_with('(') || value.ends_with(')');
    if negative_parentheses {
        if !value.starts_with('(') || !value.ends_with(')') {
            return None;
        }
        value = value[1..value.len() - 1].trim();
        if value.contains('(') || value.contains(')') {
            return None;
        }
    } else if value.contains('(') || value.contains(')') {
        return None;
    }

    let mut negative_sign = false;
    loop {
        value = value.trim_start();
        if let Some(rest) = value.strip_prefix('+') {
            value = rest;
            break;
        }
        if let Some(rest) = value.strip_prefix('-') {
            negative_sign = true;
            value = rest;
            break;
        }
        if let Some(rest) = strip_currency_prefix(value) {
            value = rest;
            continue;
        }
        break;
    }

    loop {
        value = value.trim_start();
        if let Some(rest) = strip_currency_prefix(value) {
            value = rest;
            continue;
        }
        break;
    }

    loop {
        value = value.trim_end();
        if let Some(rest) = strip_currency_suffix(value) {
            value = rest;
            continue;
        }
        break;
    }

    value = value.trim();
    if value.is_empty()
        || value
            .chars()
            .any(|ch| !ch.is_ascii_digit() && !matches!(ch, '.' | ','))
        || (negative_parentheses && negative_sign)
    {
        return None;
    }

    Some((value, negative_parentheses || negative_sign))
}

fn strip_currency_prefix(value: &str) -> Option<&str> {
    value
        .strip_prefix('£')
        .or_else(|| value.strip_prefix('$'))
        .or_else(|| value.strip_prefix('€'))
        .or_else(|| value.strip_prefix('¥'))
        .or_else(|| strip_ascii_prefix_ignore_case(value, "GBP"))
}

fn strip_currency_suffix(value: &str) -> Option<&str> {
    value
        .strip_suffix('£')
        .or_else(|| value.strip_suffix('$'))
        .or_else(|| value.strip_suffix('€'))
        .or_else(|| value.strip_suffix('¥'))
        .or_else(|| strip_ascii_suffix_ignore_case(value, "GBP"))
}

fn strip_ascii_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
        .then(|| &value[prefix.len()..])
}

fn strip_ascii_suffix_ignore_case<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let start = value.len().checked_sub(suffix.len())?;
    value
        .get(start..)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(suffix))
        .then(|| &value[..start])
}

fn valid_integer_part(value: &str, thousands_separator: char) -> bool {
    if value.contains(thousands_separator) {
        let mut groups = value.split(thousands_separator);
        let Some(first_group) = groups.next() else {
            return false;
        };
        !first_group.is_empty()
            && first_group.len() <= 3
            && first_group.chars().all(|ch| ch.is_ascii_digit())
            && groups.all(|group| group.len() == 3 && group.chars().all(|ch| ch.is_ascii_digit()))
    } else {
        value.chars().all(|ch| ch.is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{
        parse_amount_minor, parse_date, CsvSnapshotImportBalanceFormat,
        CsvSnapshotImportDateFormat, CsvSnapshotImportMissingTimezonePolicy,
        CsvSnapshotImportTimestampDatePolicy,
    };

    const COMMA_THOUSANDS: CsvSnapshotImportBalanceFormat =
        CsvSnapshotImportBalanceFormat::ThousandsCommaDecimalDot;
    const DOT_THOUSANDS: CsvSnapshotImportBalanceFormat =
        CsvSnapshotImportBalanceFormat::ThousandsDotDecimalComma;
    const ISO_8601: CsvSnapshotImportDateFormat = CsvSnapshotImportDateFormat::Iso8601DateTime;

    #[test]
    fn parse_amount_minor_accepts_expected_currency_decoration() {
        assert_eq!(
            parse_amount_minor("GBP 1,234.56", COMMA_THOUSANDS),
            Some(123456)
        );
        assert_eq!(
            parse_amount_minor("£-1,234.56", COMMA_THOUSANDS),
            Some(-123456)
        );
        assert_eq!(
            parse_amount_minor("(£1,234.56)", COMMA_THOUSANDS),
            Some(-123456)
        );
        assert_eq!(
            parse_amount_minor("1.234,56 GBP", DOT_THOUSANDS),
            Some(123456)
        );
    }

    #[test]
    fn parse_amount_minor_rejects_unexpected_text() {
        assert_eq!(parse_amount_minor("abc123", COMMA_THOUSANDS), None);
        assert_eq!(parse_amount_minor("123 pounds", COMMA_THOUSANDS), None);
        assert_eq!(parse_amount_minor("GBP abc123", COMMA_THOUSANDS), None);
    }

    #[test]
    fn parse_amount_minor_enforces_selected_balance_format() {
        assert_eq!(parse_amount_minor("1,23", COMMA_THOUSANDS), None);
        assert_eq!(parse_amount_minor("1,23", DOT_THOUSANDS), Some(123));
        assert_eq!(parse_amount_minor("1,234.56", DOT_THOUSANDS), None);
        assert_eq!(parse_amount_minor("1.234,56", COMMA_THOUSANDS), None);
    }

    #[test]
    fn parse_amount_minor_rejects_misplaced_punctuation() {
        assert_eq!(parse_amount_minor("12,34.56", COMMA_THOUSANDS), None);
        assert_eq!(parse_amount_minor("1,234.567", COMMA_THOUSANDS), None);
        assert_eq!(parse_amount_minor("1.23.4,56", DOT_THOUSANDS), None);
    }

    #[test]
    fn parse_date_accepts_timestamp_date_as_written() {
        assert_eq!(
            parse_iso_8601_date(
                "2026-01-09T23:30:00-05:00",
                CsvSnapshotImportTimestampDatePolicy::DateAsWritten,
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 9).unwrap())
        );
    }

    #[test]
    fn parse_date_can_convert_timestamp_to_utc() {
        assert_eq!(
            parse_iso_8601_date(
                "2026-01-09T23:30:00-05:00",
                CsvSnapshotImportTimestampDatePolicy::ConvertToUtc,
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap())
        );
    }

    #[test]
    fn parse_date_accepts_common_iso_8601_variants() {
        assert_eq!(
            parse_iso_8601_date(
                "20260109 2330",
                CsvSnapshotImportTimestampDatePolicy::DateAsWritten
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 9).unwrap())
        );
        assert_eq!(
            parse_iso_8601_date(
                "2026-01-09T23:30:15.123+0000",
                CsvSnapshotImportTimestampDatePolicy::DateAsWritten
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 9).unwrap())
        );
        assert_eq!(
            parse_iso_8601_date(
                "20260109T233015Z",
                CsvSnapshotImportTimestampDatePolicy::DateAsWritten
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 9).unwrap())
        );
    }

    #[test]
    fn parse_date_uses_chosen_timezone_when_converting() {
        assert_eq!(
            parse_date(
                "2026-01-09T23:30:00",
                ISO_8601,
                CsvSnapshotImportTimestampDatePolicy::ConvertToUtc,
                CsvSnapshotImportMissingTimezonePolicy::Timezone,
                "America/New_York",
            ),
            Some(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap())
        );
    }

    fn parse_iso_8601_date(
        raw: &str,
        timestamp_date_policy: CsvSnapshotImportTimestampDatePolicy,
    ) -> Option<NaiveDate> {
        parse_date(
            raw,
            ISO_8601,
            timestamp_date_policy,
            CsvSnapshotImportMissingTimezonePolicy::Local,
            "Europe/London",
        )
    }
}
