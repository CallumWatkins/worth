use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
use std::{path::Path, process::Command};

static MIGRATOR: Migrator = sqlx::migrate!("./db/migrations");
const MIGRATIONS_PATH: &str = "src-tauri/db/migrations";

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("connect to in-memory SQLite database")
}

fn git_output(arguments: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .args(arguments)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn stable_version(tag: &str) -> Option<(u64, u64, u64)> {
    let mut components = tag.strip_prefix('v')?.split('.');
    let version = (
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
    );
    components.next().is_none().then_some(version)
}

fn latest_stable_tag() -> Option<String> {
    String::from_utf8(git_output(&["tag", "--list", "v*"]))
        .expect("Git tags are UTF-8")
        .lines()
        .filter_map(|tag| stable_version(tag).map(|version| (version, tag.to_string())))
        .max_by_key(|(version, _)| *version)
        .map(|(_, tag)| tag)
}

#[tokio::test]
async fn current_migrations_apply_to_an_empty_database_and_are_idempotent() {
    let pool = in_memory_pool().await;

    MIGRATOR
        .run(&pool)
        .await
        .expect("apply current migrations to an empty database");
    MIGRATOR
        .run(&pool)
        .await
        .expect("reapply current migrations without changing the database");

    let applied_migration_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(&pool)
            .await
            .expect("count applied migrations");
    assert_eq!(applied_migration_count, MIGRATOR.iter().count() as i64);

    let account_types: Vec<String> =
        sqlx::query_scalar("SELECT name FROM account_types ORDER BY name")
            .fetch_all(&pool)
            .await
            .expect("read current account types");
    assert_eq!(
        account_types,
        [
            "cash",
            "credit_card",
            "current",
            "investment",
            "loan",
            "pension",
            "savings"
        ]
    );
}

#[tokio::test]
async fn removing_isa_preserves_accounts_snapshots_and_search() {
    let pool = in_memory_pool().await;
    sqlx::raw_sql(include_str!("../db/migrations/0001_init.sql"))
        .execute(&pool)
        .await
        .expect("create the schema before ISA removal");
    sqlx::raw_sql(
        "INSERT INTO institutions (id, name) VALUES (1, 'Test Bank');
         INSERT INTO accounts (id, name, institution_id, type_id, currency_code,
                               account_classification, opened_date, closed_date,
                               created_at, updated_at)
         SELECT id, name || ' account', 1, id, 'GBP', 'asset', '2020-01-01', NULL,
                '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z'
         FROM account_types;
         INSERT INTO accounts (name, institution_id, type_id, currency_code,
                               account_classification, closed_date)
         SELECT 'Closed Stocks & Shares ISA', 1, id, 'GBP', 'asset', '2025-01-01'
         FROM account_types WHERE name = 'isa';
         INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor)
         SELECT id, '2024-01-01', id * 100 FROM accounts;
         INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor)
         SELECT id, '2024-02-01', id * 200 FROM accounts;
         CREATE TEMP TABLE original_accounts AS SELECT * FROM accounts;
         CREATE TEMP TABLE original_snapshots AS SELECT * FROM account_balance_snapshots;
         CREATE TEMP TABLE expected_types AS
         SELECT a.id, CASE WHEN t.name = 'isa' THEN 'savings' ELSE t.name END AS name
         FROM accounts a JOIN account_types t ON t.id = a.type_id;",
    )
    .execute(&pool)
    .await
    .expect("seed active and closed accounts with balance history");

    sqlx::raw_sql(include_str!(
        "../db/migrations/0002_remove_isa_account_type.sql"
    ))
    .execute(&pool)
    .await
    .expect("remove the ISA account type");

    let remaining_isa_types: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM account_types WHERE name = 'isa'")
            .fetch_one(&pool)
            .await
            .expect("count remaining ISA types");
    assert_eq!(remaining_isa_types, 0);

    let account_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
        .fetch_one(&pool)
        .await
        .expect("count preserved accounts");
    assert_eq!(account_count, 9);

    let unexpected_accounts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
           SELECT id, name, institution_id, currency_code, account_classification,
                  opened_date, closed_date, created_at, updated_at FROM original_accounts
           EXCEPT
           SELECT id, name, institution_id, currency_code, account_classification,
                  opened_date, closed_date, created_at, updated_at FROM accounts
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("compare account details before and after migration");
    assert_eq!(unexpected_accounts, 0);

    let actual_types: Vec<(i64, String)> = sqlx::query_as(
        "SELECT a.id, t.name FROM accounts a
         JOIN account_types t ON t.id = a.type_id ORDER BY a.id",
    )
    .fetch_all(&pool)
    .await
    .expect("read migrated account types");
    let expected_types: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM expected_types ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("read expected account types");
    assert_eq!(actual_types, expected_types);

    let snapshot_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_balance_snapshots")
        .fetch_one(&pool)
        .await
        .expect("count preserved snapshots");
    assert_eq!(snapshot_count, 18);
    let missing_snapshots: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
           SELECT * FROM original_snapshots EXCEPT SELECT * FROM account_balance_snapshots
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("compare balance history before and after migration");
    assert_eq!(missing_snapshots, 0);

    let search_types: Vec<(i64, String)> = sqlx::query_as(
        "SELECT entity_id, account_type FROM search_fts WHERE kind = 'account' ORDER BY entity_id",
    )
    .fetch_all(&pool)
    .await
    .expect("read updated search entries");
    assert_eq!(search_types, expected_types);

    let savings_search_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM search_fts WHERE search_fts MATCH 'account_type:savings'",
    )
    .fetch_one(&pool)
    .await
    .expect("search for savings accounts after migration");
    assert_eq!(savings_search_count, 3);
}

#[tokio::test]
async fn current_migrations_upgrade_the_latest_release_database() {
    let Some(release_tag) = latest_stable_tag() else {
        eprintln!("No stable release tag exists; skipping the released-database upgrade check.");
        return;
    };

    let repository_root = String::from_utf8(git_output(&["rev-parse", "--show-toplevel"]))
        .expect("repository path is UTF-8");
    let repository_root = repository_root.trim();
    let released_migrations = tempfile::tempdir().expect("create released migrations directory");
    let tree = git_output(&[
        "-C",
        repository_root,
        "ls-tree",
        "-r",
        "--name-only",
        "-z",
        &release_tag,
        "--",
        MIGRATIONS_PATH,
    ]);
    let migration_files = tree
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8(path.to_vec()).expect("migration path is UTF-8"))
        .filter(|path| path.ends_with(".sql"))
        .collect::<Vec<_>>();
    assert!(
        !migration_files.is_empty(),
        "{release_tag} contains no migration files"
    );

    for file_path in migration_files {
        let relative_path = file_path
            .strip_prefix(&format!("{MIGRATIONS_PATH}/"))
            .expect("migration is inside the migrations directory");
        let destination = released_migrations.path().join(relative_path);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create released migration subdirectory");
        }
        std::fs::write(
            destination,
            git_output(&[
                "-C",
                repository_root,
                "show",
                &format!("{release_tag}:{file_path}"),
            ]),
        )
        .expect("write released migration");
    }

    let released_migrator = Migrator::new(Path::new(released_migrations.path()))
        .await
        .expect("load released migrations");
    let pool = in_memory_pool().await;

    released_migrator
        .run(&pool)
        .await
        .expect("build a database using the latest released migrations");
    MIGRATOR
        .run(&pool)
        .await
        .expect("upgrade the latest release database using current migrations");
}
