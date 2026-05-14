pub mod schema_export;

use std::str::FromStr;

use chrono::NaiveDate;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    JsonSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
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

impl AccountTypeName {
    pub fn as_str(self) -> &'static str {
        match self {
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
}

impl FromStr for AccountTypeName {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "current" => Ok(AccountTypeName::Current),
            "savings" => Ok(AccountTypeName::Savings),
            "credit_card" => Ok(AccountTypeName::CreditCard),
            "isa" => Ok(AccountTypeName::Isa),
            "investment" => Ok(AccountTypeName::Investment),
            "pension" => Ok(AccountTypeName::Pension),
            "cash" => Ok(AccountTypeName::Cash),
            "loan" => Ok(AccountTypeName::Loan),
            _ => Err("Invalid account type"),
        }
    }
}

#[crate::export_schema]
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    JsonSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum AccountClassification {
    Asset,
    Liability,
}

impl AccountClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            AccountClassification::Asset => "asset",
            AccountClassification::Liability => "liability",
        }
    }
}

impl FromStr for AccountClassification {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(AccountClassification::Asset),
            "liability" => Ok(AccountClassification::Liability),
            _ => Err("Invalid account classification"),
        }
    }
}

#[crate::export_schema]
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    JsonSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum CurrencyCode {
    GBP,
}

impl CurrencyCode {
    pub fn as_str(self) -> &'static str {
        match self {
            CurrencyCode::GBP => "GBP",
        }
    }
}

impl FromStr for CurrencyCode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GBP" => Ok(CurrencyCode::GBP),
            _ => Err("Invalid currency code"),
        }
    }
}

#[crate::export_schema]
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    JsonSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemePreference::System => "system",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }
}

impl FromStr for ThemePreference {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(ThemePreference::System),
            "light" => Ok(ThemePreference::Light),
            "dark" => Ok(ThemePreference::Dark),
            _ => Err("Invalid theme preference"),
        }
    }
}

#[crate::export_schema]
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    JsonSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum AppLocaleCode {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "en-GB")]
    EnGb,
}

impl AppLocaleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            AppLocaleCode::System => "system",
            AppLocaleCode::EnGb => "en-GB",
        }
    }
}

impl FromStr for AppLocaleCode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(AppLocaleCode::System),
            "en-GB" => Ok(AppLocaleCode::EnGb),
            _ => Err("Invalid locale code"),
        }
    }
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AppSettingsUpdateInput {
    #[garde(skip)]
    #[specta(optional)]
    pub analytics_enabled: Option<bool>,
    #[garde(skip)]
    #[specta(optional)]
    pub default_display_currency_code: Option<CurrencyCode>,
    #[garde(skip)]
    #[specta(optional)]
    pub display_locale: Option<AppLocaleCode>,
    #[garde(skip)]
    #[specta(optional)]
    pub theme: Option<ThemePreference>,
}

pub(crate) const BALANCE_MINOR_ABS_MAX: i64 = 99_999_999_999_999;

const INSTITUTION_NAME_REQUIRED: &str = "Enter an institution name";
const INSTITUTION_NAME_MAX_LENGTH: &str = "Institution name must be 80 characters or fewer";
const ACCOUNT_NAME_REQUIRED: &str = "Enter an account name";
const ACCOUNT_NAME_MAX_LENGTH: &str = "Account name must be 80 characters or fewer";
const INSTITUTION_REQUIRED: &str = "Select or create an institution";
const ACCOUNT_TYPE_REQUIRED: &str = "Select an account type";
const CURRENCY_REQUIRED: &str = "Select a currency";
const ACCOUNT_CLASSIFICATION_REQUIRED: &str = "Select a balance type";
const SNAPSHOT_REQUIRED: &str = "Add at least one snapshot";
const SNAPSHOT_SELECTION_REQUIRED: &str = "Select at least one snapshot";
const BALANCE_REQUIRED: &str = "Enter a balance";
const BALANCE_TOO_LARGE: &str = "Balance is too large";

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct InstitutionUpsertInput {
    #[garde(custom(validate_institution_name))]
    #[schemars(
        length(min = 1, max = 80),
        pattern(r".*\S.*"),
        extend("x-validation" = ::serde_json::json!({
            "required": INSTITUTION_NAME_REQUIRED,
            "blank": INSTITUTION_NAME_REQUIRED,
            "maxLength": INSTITUTION_NAME_MAX_LENGTH,
            "type": INSTITUTION_NAME_REQUIRED
        }))
    )]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
#[schemars(extend("discriminator" = ::serde_json::json!({"propertyName": "kind"})))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstitutionRef {
    Existing {
        #[garde(custom(validate_institution_id))]
        #[schemars(
            range(min = 1),
            extend("x-validation" = ::serde_json::json!({
                "required": INSTITUTION_REQUIRED,
                "minimum": INSTITUTION_REQUIRED,
                "type": INSTITUTION_REQUIRED
            }))
        )]
        id: i64,
    },
    New {
        #[garde(dive)]
        input: InstitutionUpsertInput,
    },
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountUpsertInput {
    #[garde(dive)]
    #[schemars(extend("x-validation" = ::serde_json::json!({
        "required": INSTITUTION_REQUIRED,
        "invalid": INSTITUTION_REQUIRED,
        "type": INSTITUTION_REQUIRED
    })))]
    pub institution: InstitutionRef,
    #[garde(custom(validate_account_name))]
    #[schemars(
        length(min = 1, max = 80),
        pattern(r".*\S.*"),
        extend("x-validation" = ::serde_json::json!({
            "required": ACCOUNT_NAME_REQUIRED,
            "blank": ACCOUNT_NAME_REQUIRED,
            "maxLength": ACCOUNT_NAME_MAX_LENGTH,
            "type": ACCOUNT_NAME_REQUIRED
        }))
    )]
    pub name: String,
    #[garde(skip)]
    #[schemars(extend("x-validation" = ::serde_json::json!({
        "required": ACCOUNT_TYPE_REQUIRED,
        "invalid": ACCOUNT_TYPE_REQUIRED,
        "type": ACCOUNT_TYPE_REQUIRED
    })))]
    pub account_type: AccountTypeName,
    #[garde(skip)]
    #[schemars(extend("x-validation" = ::serde_json::json!({
        "required": CURRENCY_REQUIRED,
        "invalid": CURRENCY_REQUIRED,
        "type": CURRENCY_REQUIRED
    })))]
    pub currency_code: CurrencyCode,
    #[garde(skip)]
    #[schemars(extend("x-validation" = ::serde_json::json!({
        "required": ACCOUNT_CLASSIFICATION_REQUIRED,
        "invalid": ACCOUNT_CLASSIFICATION_REQUIRED,
        "type": ACCOUNT_CLASSIFICATION_REQUIRED
    })))]
    pub account_classification: AccountClassification,
    #[garde(skip)]
    #[specta(optional)]
    pub opened_date: Option<NaiveDate>,
    #[garde(skip)]
    #[specta(optional)]
    pub closed_date: Option<NaiveDate>,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotWriteInput {
    #[garde(skip)]
    pub date: NaiveDate,
    #[garde(custom(validate_balance_minor))]
    #[schemars(
        range(min = -99999999999999i64, max = 99999999999999i64),
        extend("x-validation" = ::serde_json::json!({
            "required": BALANCE_REQUIRED,
            "minimum": BALANCE_TOO_LARGE,
            "maximum": BALANCE_TOO_LARGE,
            "type": BALANCE_REQUIRED
        }))
    )]
    pub balance_minor: i64,
    #[garde(skip)]
    pub overwrite_existing: bool,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotsCreateInput {
    #[garde(custom(validate_snapshots_non_empty), dive)]
    #[schemars(length(min = 1), extend("x-validation" = ::serde_json::json!({
        "required": SNAPSHOT_REQUIRED,
        "minItems": SNAPSHOT_REQUIRED,
        "type": SNAPSHOT_REQUIRED
    })))]
    pub snapshots: Vec<AccountSnapshotWriteInput>,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotUpdateInput {
    #[garde(skip)]
    pub date: NaiveDate,
    #[garde(custom(validate_balance_minor))]
    #[schemars(
        range(min = -99999999999999i64, max = 99999999999999i64),
        extend("x-validation" = ::serde_json::json!({
            "required": BALANCE_REQUIRED,
            "minimum": BALANCE_TOO_LARGE,
            "maximum": BALANCE_TOO_LARGE,
            "type": BALANCE_REQUIRED
        }))
    )]
    pub balance_minor: i64,
    #[garde(skip)]
    pub overwrite_existing: bool,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotsDeleteInput {
    #[garde(custom(validate_snapshot_ids_non_empty))]
    #[schemars(length(min = 1), extend("x-validation" = ::serde_json::json!({
        "required": SNAPSHOT_SELECTION_REQUIRED,
        "minItems": SNAPSHOT_SELECTION_REQUIRED,
        "type": SNAPSHOT_SELECTION_REQUIRED
    })))]
    pub snapshot_ids: Vec<i64>,
}

fn validate_institution_name(value: &str, _ctx: &()) -> garde::Result {
    validate_name(
        value,
        INSTITUTION_NAME_REQUIRED,
        INSTITUTION_NAME_MAX_LENGTH,
    )
}

fn validate_account_name(value: &str, _ctx: &()) -> garde::Result {
    validate_name(value, ACCOUNT_NAME_REQUIRED, ACCOUNT_NAME_MAX_LENGTH)
}

fn validate_name(value: &str, empty_message: &str, max_length_message: &str) -> garde::Result {
    if value.is_empty() {
        return Err(garde::Error::new(empty_message));
    }

    if value.chars().count() > 80 {
        return Err(garde::Error::new(max_length_message));
    }

    Ok(())
}

fn validate_balance_minor(value: &i64, _ctx: &()) -> garde::Result {
    if !(-BALANCE_MINOR_ABS_MAX..=BALANCE_MINOR_ABS_MAX).contains(value) {
        return Err(garde::Error::new(BALANCE_TOO_LARGE));
    }

    Ok(())
}

fn validate_institution_id(value: &i64, _ctx: &()) -> garde::Result {
    if *value < 1 {
        return Err(garde::Error::new(INSTITUTION_REQUIRED));
    }

    Ok(())
}

fn validate_snapshots_non_empty(value: &[AccountSnapshotWriteInput], _ctx: &()) -> garde::Result {
    if value.is_empty() {
        return Err(garde::Error::new(SNAPSHOT_REQUIRED));
    }

    Ok(())
}

fn validate_snapshot_ids_non_empty(value: &[i64], _ctx: &()) -> garde::Result {
    if value.is_empty() {
        return Err(garde::Error::new(SNAPSHOT_SELECTION_REQUIRED));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use garde::Validate;

    use super::{
        AccountSnapshotUpdateInput, AccountSnapshotWriteInput, BALANCE_MINOR_ABS_MAX,
        BALANCE_TOO_LARGE, InstitutionUpsertInput,
    };

    #[test]
    fn snapshot_write_inputs_reject_balances_above_javascript_safe_integer_range() {
        let input = AccountSnapshotWriteInput {
            date: date(),
            balance_minor: BALANCE_MINOR_ABS_MAX + 1,
            overwrite_existing: false,
        };

        assert_validation_message(input.validate(), BALANCE_TOO_LARGE);
    }

    #[test]
    fn snapshot_update_inputs_reject_balances_below_javascript_safe_integer_range() {
        let input = AccountSnapshotUpdateInput {
            date: date(),
            balance_minor: -BALANCE_MINOR_ABS_MAX - 1,
            overwrite_existing: false,
        };

        assert_validation_message(input.validate(), BALANCE_TOO_LARGE);
    }

    #[test]
    fn name_lengths_are_counted_as_unicode_scalar_values() {
        let input = InstitutionUpsertInput {
            name: "💷".repeat(80),
        };

        assert!(input.validate().is_ok());

        let input = InstitutionUpsertInput {
            name: "💷".repeat(81),
        };

        assert_validation_message(
            input.validate(),
            "Institution name must be 80 characters or fewer",
        );
    }

    fn assert_validation_message(result: Result<(), garde::error::Report>, message: &str) {
        let Err(report) = result else {
            panic!("expected validation error");
        };

        assert!(
            report
                .iter()
                .any(|(_path, error)| error.to_string() == message)
        );
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 9).unwrap()
    }
}
