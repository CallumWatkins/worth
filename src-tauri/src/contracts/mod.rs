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

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
pub struct AccountUpsertInput {
    #[garde(dive)]
    pub institution: InstitutionRef,
    #[garde(length(min = 1, max = 120), pattern(r".*\S.*"))]
    pub name: String,
    #[garde(skip)]
    pub account_type: AccountTypeName,
    #[garde(length(equal = 3), pattern(r"^[A-Z]{3}$"))]
    pub currency_code: String,
    #[garde(custom(validate_normal_balance_sign))]
    #[schemars(extend("enum" = [-1, 1]))]
    pub normal_balance_sign: i32,
    #[garde(skip)]
    pub opened_date: Option<NaiveDate>,
}

fn validate_normal_balance_sign(value: &i32, _ctx: &()) -> garde::Result {
    if !matches!(*value, -1 | 1) {
        return Err(garde::Error::new("Normal balance sign must be 1 or -1"));
    }
    Ok(())
}
