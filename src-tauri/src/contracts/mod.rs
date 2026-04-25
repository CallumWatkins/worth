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

#[crate::export_schema]
#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct InstitutionUpsertInput {
    #[garde(length(min = 1, max = 120), pattern(r".*\S.*"))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
#[schemars(extend("discriminator" = {"propertyName": "kind"}))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstitutionRef {
    Existing {
        #[garde(range(min = 1))]
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
    pub institution: InstitutionRef,
    #[garde(length(min = 1, max = 120), pattern(r".*\S.*"))]
    pub name: String,
    #[garde(skip)]
    pub account_type: AccountTypeName,
    #[garde(skip)]
    pub currency_code: CurrencyCode,
    #[garde(custom(validate_normal_balance_sign))]
    #[schemars(extend("enum" = [-1, 1]))]
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
    #[garde(length(min = 1), dive)]
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
    #[garde(length(min = 1))]
    pub snapshot_ids: Vec<i64>,
}

fn validate_normal_balance_sign(value: &i32, _ctx: &()) -> garde::Result {
    if !matches!(*value, -1 | 1) {
        return Err(garde::Error::new("Normal balance sign must be 1 or -1"));
    }
    Ok(())
}
