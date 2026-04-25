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

const INSTITUTION_NAME_REQUIRED: &str = "Enter an institution name";
const INSTITUTION_NAME_MAX_LENGTH: &str = "Institution name must be 120 characters or fewer";
const ACCOUNT_NAME_REQUIRED: &str = "Enter an account name";
const ACCOUNT_NAME_MAX_LENGTH: &str = "Account name must be 120 characters or fewer";
const INSTITUTION_REQUIRED: &str = "Select or create an institution";
const ACCOUNT_TYPE_REQUIRED: &str = "Select an account type";
const CURRENCY_REQUIRED: &str = "Select a currency";
const NORMAL_BALANCE_SIGN_REQUIRED: &str =
    "Select whether this account normally has a positive or negative balance";
const SNAPSHOT_REQUIRED: &str = "Add at least one snapshot";
const SNAPSHOT_SELECTION_REQUIRED: &str = "Select at least one snapshot";

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct InstitutionUpsertInput {
    #[garde(custom(validate_institution_name))]
    #[schemars(
        length(min = 1, max = 120),
        pattern(r".*\S.*"),
        extend("x-validation" = {
            "required": INSTITUTION_NAME_REQUIRED,
            "blank": INSTITUTION_NAME_REQUIRED,
            "maxLength": INSTITUTION_NAME_MAX_LENGTH,
            "type": INSTITUTION_NAME_REQUIRED
        })
    )]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
#[schemars(extend("discriminator" = {"propertyName": "kind"}))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstitutionRef {
    Existing {
        #[garde(custom(validate_institution_id))]
        #[schemars(
            range(min = 1),
            extend("x-validation" = {
                "required": INSTITUTION_REQUIRED,
                "minimum": INSTITUTION_REQUIRED,
                "type": INSTITUTION_REQUIRED
            })
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
    #[schemars(extend("x-validation" = {
        "required": INSTITUTION_REQUIRED,
        "invalid": INSTITUTION_REQUIRED,
        "type": INSTITUTION_REQUIRED
    }))]
    pub institution: InstitutionRef,
    #[garde(custom(validate_account_name))]
    #[schemars(
        length(min = 1, max = 120),
        pattern(r".*\S.*"),
        extend("x-validation" = {
            "required": ACCOUNT_NAME_REQUIRED,
            "blank": ACCOUNT_NAME_REQUIRED,
            "maxLength": ACCOUNT_NAME_MAX_LENGTH,
            "type": ACCOUNT_NAME_REQUIRED
        })
    )]
    pub name: String,
    #[garde(skip)]
    #[schemars(extend("x-validation" = {
        "required": ACCOUNT_TYPE_REQUIRED,
        "invalid": ACCOUNT_TYPE_REQUIRED,
        "type": ACCOUNT_TYPE_REQUIRED
    }))]
    pub account_type: AccountTypeName,
    #[garde(skip)]
    #[schemars(extend("x-validation" = {
        "required": CURRENCY_REQUIRED,
        "invalid": CURRENCY_REQUIRED,
        "type": CURRENCY_REQUIRED
    }))]
    pub currency_code: CurrencyCode,
    #[garde(custom(validate_normal_balance_sign))]
    #[schemars(extend(
        "enum" = [-1, 1],
        "x-validation" = {
            "required": NORMAL_BALANCE_SIGN_REQUIRED,
            "invalid": NORMAL_BALANCE_SIGN_REQUIRED,
            "type": NORMAL_BALANCE_SIGN_REQUIRED
        }
    ))]
    pub normal_balance_sign: i32,
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
    #[garde(skip)]
    pub balance_minor: i64,
    #[garde(skip)]
    pub overwrite_existing: bool,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotsCreateInput {
    #[garde(custom(validate_snapshots_non_empty), dive)]
    #[schemars(length(min = 1), extend("x-validation" = {
        "required": SNAPSHOT_REQUIRED,
        "minItems": SNAPSHOT_REQUIRED,
        "type": SNAPSHOT_REQUIRED
    }))]
    pub snapshots: Vec<AccountSnapshotWriteInput>,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotUpdateInput {
    #[garde(skip)]
    pub date: NaiveDate,
    #[garde(skip)]
    pub balance_minor: i64,
    #[garde(skip)]
    pub overwrite_existing: bool,
}

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountSnapshotsDeleteInput {
    #[garde(custom(validate_snapshot_ids_non_empty))]
    #[schemars(length(min = 1), extend("x-validation" = {
        "required": SNAPSHOT_SELECTION_REQUIRED,
        "minItems": SNAPSHOT_SELECTION_REQUIRED,
        "type": SNAPSHOT_SELECTION_REQUIRED
    }))]
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

    if value.len() > 120 {
        return Err(garde::Error::new(max_length_message));
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

fn validate_normal_balance_sign(value: &i32, _ctx: &()) -> garde::Result {
    if !matches!(*value, -1 | 1) {
        return Err(garde::Error::new(NORMAL_BALANCE_SIGN_REQUIRED));
    }
    Ok(())
}
