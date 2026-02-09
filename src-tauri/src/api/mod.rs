use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use chrono::{Duration, NaiveDate, Utc};
use std::collections::BTreeMap;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum ApiError {
    #[error("Database error")]
    Db,
    #[error("Not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord, Hash,
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
    pub opened_date: Option<String>,
    pub closed_date: Option<String>,
    pub first_snapshot_date: String,  // "YYYY-MM-DD"
    pub latest_snapshot_date: String, // "YYYY-MM-DD"
    pub latest_balance_minor: i64,
    pub activity_by_period: BTreeMap<ActivityPeriod, ActivityDataDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardAllocationDto {
    pub account_type: AccountTypeName,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardBalancePointDto {
    pub date: String, // "YYYY-MM-DD"
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DashboardDto {
    pub total_balance_minor: i64,
    pub change_vs_last_month_pct: f64,
    pub monthly_yield_minor: i64,
    pub active_accounts: u32,
    pub allocation_by_type: Vec<DashboardAllocationDto>,
    pub balance_over_time: Vec<DashboardBalancePointDto>,
}

#[tauri::command]
#[specta::specta]
pub async fn accounts_list() -> Result<Vec<AccountDto>, ApiError> {
    Ok(dummy_accounts_full())
}

#[tauri::command]
#[specta::specta]
pub async fn dashboard_get() -> Result<DashboardDto, ApiError> {
    Ok(dummy_dashboard())
}

#[tauri::command]
#[specta::specta]
pub async fn dashboard_balance_over_time(
    period: BalanceOverTimePeriod,
) -> Result<Vec<DashboardBalancePointDto>, ApiError> {
    let today = Utc::now().date_naive();
    let base = dummy_accounts_base();
    let total_balance_minor: i64 = base.iter().map(|a| a.latest_balance_minor).sum();

    let days = match period {
        BalanceOverTimePeriod::P1M => 30,
        BalanceOverTimePeriod::P6M => 183,
        BalanceOverTimePeriod::P1Y => 365,
        BalanceOverTimePeriod::Max => 365 * 5,
    };

    Ok(generate_dashboard_series(today, days, total_balance_minor))
}

#[derive(Debug, Clone, Copy)]
struct DummyInstitution {
    id: i64,
    name: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct DummyAccountType {
    id: i64,
    name: AccountTypeName,
}

#[derive(Debug, Clone, Copy)]
struct BaseAccount {
    id: i64,
    name: &'static str,
    institution: DummyInstitution,
    account_type: DummyAccountType,
    currency_code: &'static str,
    normal_balance_sign: i32,
    opened_date: Option<&'static str>,
    closed_date: Option<&'static str>,
    latest_balance_minor: i64,
}

fn iso(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn dummy_accounts_base() -> Vec<BaseAccount> {
    let nationwide = DummyInstitution {
        id: 1,
        name: "Nationwide",
    };
    let monzo = DummyInstitution {
        id: 2,
        name: "Monzo",
    };
    let trading212 = DummyInstitution {
        id: 3,
        name: "Trading 212",
    };
    let aviva = DummyInstitution {
        id: 4,
        name: "Aviva",
    };
    let starling = DummyInstitution {
        id: 5,
        name: "Starling",
    };
    let hsbc = DummyInstitution {
        id: 6,
        name: "HSBC",
    };
    let amex = DummyInstitution {
        id: 7,
        name: "American Express",
    };

    let current = DummyAccountType {
        id: 1,
        name: AccountTypeName::Current,
    };
    let savings = DummyAccountType {
        id: 2,
        name: AccountTypeName::Savings,
    };
    let credit_card = DummyAccountType {
        id: 3,
        name: AccountTypeName::CreditCard,
    };
    let isa = DummyAccountType {
        id: 4,
        name: AccountTypeName::Isa,
    };
    let investment = DummyAccountType {
        id: 5,
        name: AccountTypeName::Investment,
    };
    let pension = DummyAccountType {
        id: 6,
        name: AccountTypeName::Pension,
    };

    vec![
        BaseAccount {
            id: 1,
            name: "Everyday Current",
            institution: nationwide,
            account_type: current,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 243_512,
        },
        BaseAccount {
            id: 2,
            name: "Bills Pot",
            institution: monzo,
            account_type: current,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 0,
        },
        BaseAccount {
            id: 3,
            name: "Rainy Day Savings",
            institution: nationwide,
            account_type: savings,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 1_325_000,
        },
        BaseAccount {
            id: 4,
            name: "Emergency Fund",
            institution: nationwide,
            account_type: savings,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 800_000,
        },
        BaseAccount {
            id: 5,
            name: "Stocks & Shares ISA",
            institution: trading212,
            account_type: isa,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 4_589_042,
        },
        BaseAccount {
            id: 6,
            name: "General Investment Account",
            institution: trading212,
            account_type: investment,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 1_211_070,
        },
        BaseAccount {
            id: 7,
            name: "Workplace Pension",
            institution: aviva,
            account_type: pension,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 9_802_533,
        },
        BaseAccount {
            id: 8,
            name: "Holiday Savings",
            institution: starling,
            account_type: savings,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 142_000,
        },
        BaseAccount {
            id: 9,
            name: "Cash ISA (Legacy)",
            institution: hsbc,
            account_type: isa,
            currency_code: "GBP",
            normal_balance_sign: 1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: 0,
        },
        BaseAccount {
            id: 10,
            name: "Everyday Credit Card",
            institution: amex,
            account_type: credit_card,
            currency_code: "GBP",
            normal_balance_sign: -1,
            opened_date: None,
            closed_date: None,
            latest_balance_minor: -54_321,
        },
    ]
}

fn dummy_accounts_full() -> Vec<AccountDto> {
    let today = Utc::now().date_naive();

    dummy_accounts_base()
        .into_iter()
        .map(|base| {
            let (history_start, history_minor) = generate_account_history(
                base.id,
                base.account_type.name,
                base.latest_balance_minor,
                base.normal_balance_sign,
                today,
            );

            let periods: [(ActivityPeriod, usize); 4] = [
                (ActivityPeriod::P1W, 7),
                (ActivityPeriod::P1M, 30),
                (ActivityPeriod::P3M, 90),
                (ActivityPeriod::P6M, 180),
            ];
            let mut activity_by_period = BTreeMap::new();

            for (key, points) in periods {
                let period_start = today - Duration::days(points as i64 - 1);
                let values = values_for_period(history_start, &history_minor, period_start, points);
                let delta_minor = delta_from_values(&values);
                activity_by_period.insert(
                    key,
                    ActivityDataDto {
                        values,
                        delta_minor,
                    },
                );
            }

            AccountDto {
                id: base.id,
                name: base.name.to_string(),
                institution: InstitutionDto {
                    id: base.institution.id,
                    name: base.institution.name.to_string(),
                },
                account_type: AccountTypeDto {
                    id: base.account_type.id,
                    name: base.account_type.name,
                },
                currency_code: base.currency_code.to_string(),
                normal_balance_sign: base.normal_balance_sign,
                opened_date: base.opened_date.map(|d| d.to_string()),
                closed_date: base.closed_date.map(|d| d.to_string()),
                first_snapshot_date: iso(history_start),
                latest_snapshot_date: iso(today),
                latest_balance_minor: base.latest_balance_minor,
                activity_by_period,
            }
        })
        .collect()
}

fn dummy_dashboard() -> DashboardDto {
    let today = Utc::now().date_naive();
    let base = dummy_accounts_base();

    let total_balance_minor: i64 = base.iter().map(|a| a.latest_balance_minor).sum();
    let active_accounts: u32 = base.iter().filter(|a| a.latest_balance_minor != 0).count() as u32;

    let mut allocation_map: BTreeMap<AccountTypeName, i64> = BTreeMap::new();
    for a in &base {
        *allocation_map.entry(a.account_type.name).or_insert(0) += a.latest_balance_minor;
    }

    // Pie chart allocations should be non-negative; exclude net-negative/zero groups (e.g. credit cards).
    let allocation_by_type = allocation_map
        .into_iter()
        .filter(|(_kind, balance_minor)| *balance_minor > 0)
        .map(|(account_type, balance_minor)| DashboardAllocationDto {
            account_type,
            balance_minor,
        })
        .collect::<Vec<_>>();

    let balance_over_time = generate_dashboard_series(today, 183, total_balance_minor);

    let last_minor = balance_over_time
        .last()
        .map(|p| p.balance_minor)
        .unwrap_or(total_balance_minor);

    let month_ago_minor = if balance_over_time.len() >= 31 {
        balance_over_time[balance_over_time.len() - 31].balance_minor
    } else {
        last_minor
    };

    let monthly_yield_minor = last_minor - month_ago_minor;
    let change_vs_last_month_pct = if month_ago_minor != 0 {
        (monthly_yield_minor as f64) / (month_ago_minor as f64) * 100.0
    } else {
        0.0
    };

    DashboardDto {
        total_balance_minor,
        change_vs_last_month_pct,
        monthly_yield_minor,
        active_accounts,
        allocation_by_type,
        balance_over_time,
    }
}

fn generate_dashboard_series(
    today: NaiveDate,
    days: usize,
    target_last_minor: i64,
) -> Vec<DashboardBalancePointDto> {
    if days == 0 {
        return vec![];
    }

    let target_last_pounds = target_last_minor as f64 / 100.0;
    let raw_last = dashboard_base_pounds(today);
    let shift = target_last_pounds - raw_last;

    let mut out = Vec::with_capacity(days);
    for i in 0..days {
        let offset_days = (days - 1).saturating_sub(i) as i64;
        let date = today - Duration::days(offset_days);
        let pounds = dashboard_base_pounds(date) + shift;
        out.push(DashboardBalancePointDto {
            date: iso(date),
            balance_minor: round2_to_minor(pounds),
        });
    }

    out
}

fn dashboard_base_pounds(date: NaiveDate) -> f64 {
    // Fixed origin means overlapping dates produce identical values across requested ranges.
    let origin = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let t = (date - origin).num_days() as f64;
    245_000.0 + t * 35.0 + (t / 9.0).sin() * 900.0 + (t / 21.0).cos() * 500.0
}

fn round2_to_minor(value_pounds: f64) -> i64 {
    (value_pounds * 100.0).round() as i64
}

fn values_for_period(
    history_start: NaiveDate,
    history_minor: &[i64],
    period_start: NaiveDate,
    points: usize,
) -> Vec<Option<i64>> {
    if points == 0 {
        return vec![];
    }

    let start_index = (period_start - history_start).num_days();
    let (missing, slice_start) = if start_index < 0 {
        ((-start_index) as usize, 0usize)
    } else {
        (0usize, start_index as usize)
    };

    let slice_len = points.saturating_sub(missing);
    let mut out = Vec::with_capacity(points);

    out.extend(std::iter::repeat_n(None, missing));
    out.extend(
        history_minor
            .iter()
            .skip(slice_start)
            .take(slice_len)
            .map(|v| Some(*v)),
    );

    out
}

fn delta_from_values(values: &[Option<i64>]) -> i64 {
    let first = values.iter().copied().flatten().next();
    let last = values.last().copied().flatten();

    match (first, last) {
        (Some(f), Some(l)) => l - f,
        _ => 0,
    }
}

fn history_range_days(account_type: AccountTypeName) -> (i32, i32) {
    match account_type {
        AccountTypeName::Current => (7, 120),
        AccountTypeName::Savings => (14, 220),
        AccountTypeName::CreditCard => (7, 120),
        AccountTypeName::Isa => (30, 220),
        AccountTypeName::Investment => (14, 220),
        AccountTypeName::Pension => (90, 220),
        AccountTypeName::Cash => (7, 120),
        AccountTypeName::Loan => (30, 220),
    }
}

fn volatility(account_type: AccountTypeName) -> f64 {
    match account_type {
        AccountTypeName::Current => 0.02,
        AccountTypeName::Savings => 0.006,
        AccountTypeName::CreditCard => 0.025,
        AccountTypeName::Isa => 0.02,
        AccountTypeName::Investment => 0.03,
        AccountTypeName::Pension => 0.01,
        AccountTypeName::Cash => 0.004,
        AccountTypeName::Loan => 0.002,
    }
}

fn generate_account_history(
    account_id: i64,
    account_type: AccountTypeName,
    latest_balance_minor: i64,
    normal_balance_sign: i32,
    today: NaiveDate,
) -> (NaiveDate, Vec<i64>) {
    let seed = fnv1a_32(&format!("account:{account_id}"));
    let mut rng = Mulberry32::new(seed);

    let (min, max) = history_range_days(account_type);
    let days_ago = rand_int(&mut rng, min, max);

    let start_date = today - Duration::days(days_ago as i64);
    let points = (days_ago + 1) as usize;

    let mut balances_minor = vec![0i64; points];
    if points == 0 {
        return (start_date, balances_minor);
    }

    balances_minor[points - 1] = latest_balance_minor;

    let current_pounds = latest_balance_minor as f64 / 100.0;
    let scale = current_pounds.abs().max(1000.0);
    let vol = volatility(account_type);

    for i in (0..(points - 1)).rev() {
        let noise = (rng.next_f64() - 0.5) * 2.0;
        let delta = noise * vol * scale;

        let next_pounds = balances_minor[i + 1] as f64 / 100.0;
        let prev_pounds = next_pounds - delta;
        let prev_pounds = if normal_balance_sign >= 0 {
            prev_pounds.max(0.0)
        } else {
            prev_pounds.min(0.0)
        };
        balances_minor[i] = round2_to_minor(prev_pounds);
    }

    (start_date, balances_minor)
}

fn fnv1a_32(s: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for b in s.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

fn rand_int(rng: &mut Mulberry32, min: i32, max: i32) -> i32 {
    if min >= max {
        return min;
    }

    let span = (max - min + 1) as f64;
    let n = (rng.next_f64() * span).floor() as i32;
    min + n
}

struct Mulberry32 {
    a: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self { a: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6D2B79F5);

        let mut t = imul(self.a ^ (self.a >> 15), self.a | 1);
        t = (t.wrapping_add(imul(t ^ (t >> 7), t | 61))) ^ t;
        let out = t ^ (t >> 14);

        (out as f64) / 4_294_967_296.0
    }
}

fn imul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    use tauri_specta::{collect_commands, Builder};

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        accounts_list,
        dashboard_get,
        dashboard_balance_over_time,
    ]);

    #[cfg(debug_assertions)]
    {
        use std::path::PathBuf;

        let bindings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("app")
            .join("bindings.ts");

        builder
            .export(
                Typescript::default().bigint(BigIntExportBehavior::Number),
                bindings_path,
            )
            .expect("Failed to export typescript bindings");
    }

    builder.invoke_handler()
}
