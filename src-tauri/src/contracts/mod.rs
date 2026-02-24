use chrono::NaiveDate;
use garde::{Error, Validate};
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
    #[garde(custom(validate_name))]
    #[schemars(schema_with = "name_schema")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InstitutionRef {
    Existing {
        #[garde(custom(validate_positive_id))]
        #[schemars(range(min = 1))]
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
    #[garde(custom(validate_name))]
    #[schemars(schema_with = "name_schema")]
    pub name: String,
    #[garde(skip)]
    pub account_type: AccountTypeName,
    #[garde(custom(validate_currency_code))]
    #[schemars(schema_with = "currency_code_schema")]
    pub currency_code: String,
    #[garde(custom(validate_normal_balance_sign))]
    #[schemars(schema_with = "normal_balance_sign_schema")]
    pub normal_balance_sign: i32,
    #[garde(skip)]
    pub opened_date: Option<NaiveDate>,
}

fn validate_name(value: &str, _ctx: &()) -> garde::Result {
    if value.is_empty() {
        return Err(Error::new("Name is required"));
    }
    if value.len() > 120 {
        return Err(Error::new("Name is too long (max 120 characters)"));
    }
    Ok(())
}

fn validate_currency_code(value: &str, _ctx: &()) -> garde::Result {
    let valid = value.len() == 3 && value.chars().all(|ch| ch.is_ascii_alphabetic());
    if !valid {
        return Err(Error::new(
            "Currency code must be a 3-letter code (e.g. GBP)",
        ));
    }
    Ok(())
}

fn validate_normal_balance_sign(value: &i32, _ctx: &()) -> garde::Result {
    if !matches!(*value, -1 | 1) {
        return Err(Error::new("Normal balance sign must be 1 or -1"));
    }
    Ok(())
}

fn validate_positive_id(value: &i64, _ctx: &()) -> garde::Result {
    if *value < 1 {
        return Err(Error::new("Institution id must be positive"));
    }
    Ok(())
}

fn normal_balance_sign_schema(
    _gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "integer",
        "oneOf": [
            { "const": -1 },
            { "const": 1 }
        ]
    }))
    .expect("normal balance sign schema should be valid")
}

fn name_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "string",
        "minLength": 1,
        "maxLength": 120,
        "pattern": ".*\\S.*"
    }))
    .expect("name schema should be valid")
}

fn currency_code_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "string",
        "minLength": 3,
        "maxLength": 3,
        "pattern": "^[A-Z]{3}$"
    }))
    .expect("currency code schema should be valid")
}
