#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InstitutionRow {
    pub id: i64,
    pub name: String, // unique
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountTypeRow {
    pub id: i64,
    pub name: String, // unique, e.g. "current"
}

#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    pub id: i64,
    pub name: String,
    pub institution_id: i64,
    pub type_id: i64,
    pub currency_code: String,
    pub normal_balance_sign: i32, // CHECK (normal_balance_sign IN (1, -1))
    pub opened_date: Option<String>,
    pub closed_date: Option<String>,
    pub created_at: String, // ISO8601 datetime: "YYYY-MM-DDTHH:MM:SS"
    pub updated_at: String, // ISO8601 datetime: "YYYY-MM-DDTHH:MM:SS"
}

#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountBalanceSnapshotRow {
    pub id: i64,
    pub account_id: i64,
    pub balance_date: String, // ISO8601 date: "YYYY-MM-DD"
    pub balance_minor: i64,
    pub created_at: String, // ISO8601 datetime: "YYYY-MM-DDTHH:MM:SS"
}
