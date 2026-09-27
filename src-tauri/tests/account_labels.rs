use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

const SEED_ACCOUNTS: &str = "
    INSERT INTO institutions (id, name) VALUES (1, 'Test Bank'), (2, 'Other Bank');
    INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
    SELECT 1, 'Everyday', 1, id, 'GBP', 'asset' FROM account_types WHERE name = 'current';
    INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
    SELECT 2, 'Savings pot', 2, id, 'GBP', 'asset' FROM account_types WHERE name = 'savings';
    INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor)
    VALUES (1, '2026-01-01', 10000), (2, '2026-01-01', 20000);
";

async fn in_memory_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(":memory:")
                .foreign_keys(true),
        )
        .await
        .expect("connect to in-memory SQLite database")
}

async fn seeded_pool() -> SqlitePool {
    let pool = in_memory_pool().await;
    sqlx::migrate!("./db/migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    sqlx::raw_sql(SEED_ACCOUNTS)
        .execute(&pool)
        .await
        .expect("seed accounts and snapshots");
    pool
}

async fn search_accounts(pool: &SqlitePool, query: &str) -> Vec<i64> {
    sqlx::query_scalar(
        "SELECT entity_id FROM search_fts
         WHERE search_fts MATCH ? AND kind = 'account'
           AND rank MATCH 'bm25(0.0, 0.0, 10.0, 2.0, 1.0, 1.0)'
         ORDER BY entity_id",
    )
    .bind(query)
    .fetch_all(pool)
    .await
    .expect("search indexed accounts")
}

#[tokio::test]
async fn migration_preserves_existing_accounts_balances_and_search() -> anyhow::Result<()> {
    let pool = in_memory_pool().await;
    sqlx::raw_sql(concat!(
        include_str!("../db/migrations/0001_init.sql"),
        include_str!("../db/migrations/0002_remove_isa_account_type.sql"),
        include_str!("../db/migrations/0003_account_dashboard_inclusion.sql"),
    ))
    .execute(&pool)
    .await?;
    sqlx::raw_sql(SEED_ACCOUNTS).execute(&pool).await?;
    sqlx::raw_sql(
        "CREATE TEMP TABLE original_accounts AS SELECT * FROM accounts;
         CREATE TEMP TABLE original_snapshots AS SELECT * FROM account_balance_snapshots;
         CREATE TEMP TABLE original_search AS SELECT * FROM search_fts;",
    )
    .execute(&pool)
    .await?;

    sqlx::raw_sql(include_str!("../db/migrations/0004_account_labels.sql"))
        .execute(&pool)
        .await?;

    let unchanged: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM (SELECT * FROM original_accounts EXCEPT SELECT * FROM accounts)),
           (SELECT COUNT(*) FROM (SELECT * FROM accounts EXCEPT SELECT * FROM original_accounts)),
           (SELECT COUNT(*) FROM (SELECT * FROM original_snapshots EXCEPT SELECT * FROM account_balance_snapshots)),
           (SELECT COUNT(*) FROM (SELECT * FROM account_balance_snapshots EXCEPT SELECT * FROM original_snapshots)),
           (SELECT COUNT(*) FROM (SELECT * FROM original_search EXCEPT SELECT kind, entity_id, name, institution_name, account_type FROM search_fts)),
           (SELECT COUNT(*) FROM (SELECT kind, entity_id, name, institution_name, account_type FROM search_fts EXCEPT SELECT * FROM original_search))",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(unchanged, (0, 0, 0, 0, 0, 0));
    assert_eq!(search_accounts(&pool, "Every*").await, [1]);
    assert_eq!(search_accounts(&pool, "institution_name:Other").await, [2]);
    assert_eq!(search_accounts(&pool, "account_type:savings").await, [2]);
    let label_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM labels")
        .fetch_one(&pool)
        .await?;
    assert_eq!(label_count, 0);
    Ok(())
}

#[tokio::test]
async fn label_constraints_enforce_lengths_and_unique_keys() -> anyhow::Result<()> {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "INSERT INTO labels (name, name_key) VALUES
           ('A', 'a'), ('12345678901234567890', '12345678901234567890'),
           ('ISA', 'isa'), ('Épargne', 'épargne');",
    )
    .execute(&pool)
    .await?;
    // SQLite counts Unicode characters, not UTF-8 bytes, for the display-name limit.
    let unicode_name = "界".repeat(20);
    sqlx::query("INSERT INTO labels (name, name_key) VALUES (?, ?)")
        .bind(&unicode_name)
        .bind(&unicode_name)
        .execute(&pool)
        .await?;

    let invalid = [
        ("", "empty"),
        ("123456789012345678901", "too-long"),
        (" ISA", "leading-space"),
        ("ISA ", "trailing-space"),
        ("Keyless", ""),
        ("isa", "isa"),
        ("ÉPARGNE", "épargne"),
    ];
    // The application supplies normalized keys; the database enforces their uniqueness.
    for (name, key) in invalid {
        let error = sqlx::query("INSERT INTO labels (name, name_key) VALUES (?, ?)")
            .bind(name)
            .bind(key)
            .execute(&pool)
            .await
            .expect_err("reject invalid label");
        assert!(error.as_database_error().is_some());
    }
    Ok(())
}

#[tokio::test]
async fn assignments_are_unique_and_deletion_preserves_unrelated_data() -> anyhow::Result<()> {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "INSERT INTO labels (id, name, name_key) VALUES (1, 'ISA', 'isa'), (2, 'Long term', 'long term');
         INSERT INTO account_labels (account_id, label_id) VALUES (1, 1), (2, 1), (2, 2);",
    )
    .execute(&pool)
    .await?;
    let duplicate = sqlx::query("INSERT INTO account_labels VALUES (1, 1)")
        .execute(&pool)
        .await
        .expect_err("reject duplicate assignment");
    assert!(duplicate.as_database_error().unwrap().is_unique_violation());
    let missing_label = sqlx::query("INSERT INTO account_labels VALUES (1, 999)")
        .execute(&pool)
        .await
        .expect_err("reject missing label");
    assert!(
        missing_label
            .as_database_error()
            .unwrap()
            .is_foreign_key_violation()
    );
    let missing_account = sqlx::query("INSERT INTO account_labels VALUES (999, 1)")
        .execute(&pool)
        .await
        .expect_err("reject missing account");
    assert!(
        missing_account
            .as_database_error()
            .unwrap()
            .is_foreign_key_violation()
    );

    sqlx::query("DELETE FROM labels WHERE id = 1")
        .execute(&pool)
        .await?;
    let preserved: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM accounts), (SELECT COUNT(*) FROM account_balance_snapshots),
                (SELECT SUM(balance_minor) FROM account_balance_snapshots),
                (SELECT COUNT(*) FROM account_labels)",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(preserved, (2, 2, 30000, 1));
    assert!(search_accounts(&pool, "ISA").await.is_empty());
    assert_eq!(search_accounts(&pool, "labels:Long").await, [2]);

    sqlx::query("DELETE FROM accounts WHERE id = 2")
        .execute(&pool)
        .await?;
    let retained: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM labels), (SELECT COUNT(*) FROM account_labels)",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(retained, (1, 0));
    assert!(search_accounts(&pool, "Long").await.is_empty());

    sqlx::query("INSERT INTO account_labels VALUES (1, 2)")
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM institutions WHERE id = 1")
        .execute(&pool)
        .await?;
    let retained: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM labels), (SELECT COUNT(*) FROM account_labels),
                (SELECT COUNT(*) FROM search_fts WHERE kind = 'account')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(retained, (1, 0, 0));
    Ok(())
}

#[tokio::test]
async fn search_tracks_label_assignments_renames_and_account_metadata() -> anyhow::Result<()> {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "INSERT INTO labels (id, name, name_key) VALUES (1, 'ISA', 'isa'), (2, 'Long term', 'long term');
         INSERT INTO account_labels VALUES (1, 1), (2, 1), (2, 2);",
    )
    .execute(&pool)
    .await?;
    assert_eq!(search_accounts(&pool, "IS*").await, [1, 2]);
    assert_eq!(search_accounts(&pool, "ISA Long").await, [2]);

    sqlx::query("UPDATE labels SET name = 'Tax wrapper', name_key = 'tax wrapper' WHERE id = 1")
        .execute(&pool)
        .await?;
    assert!(search_accounts(&pool, "ISA").await.is_empty());
    assert_eq!(search_accounts(&pool, "Tax wrap*").await, [1, 2]);

    sqlx::raw_sql(
        "UPDATE institutions SET name = 'Renamed Bank' WHERE id = 1;
         UPDATE account_types SET name = 'daily' WHERE name = 'current';
         UPDATE accounts SET name = 'Renamed account', institution_id = 1 WHERE id = 2;",
    )
    .execute(&pool)
    .await?;
    assert_eq!(search_accounts(&pool, "Tax Renamed").await, [1, 2]);
    assert_eq!(search_accounts(&pool, "Tax daily").await, [1]);
    assert_eq!(search_accounts(&pool, "Tax Long").await, [2]);
    assert!(
        search_accounts(&pool, "institution_name:Other")
            .await
            .is_empty()
    );
    assert!(
        search_accounts(&pool, "account_type:current")
            .await
            .is_empty()
    );

    sqlx::query("DELETE FROM account_labels WHERE account_id = 1 AND label_id = 1")
        .execute(&pool)
        .await?;
    assert_eq!(search_accounts(&pool, "Tax").await, [2]);
    sqlx::query("UPDATE account_labels SET account_id = 1 WHERE account_id = 2 AND label_id = 2")
        .execute(&pool)
        .await?;
    assert_eq!(search_accounts(&pool, "Long").await, [1]);
    assert_eq!(search_accounts(&pool, "Tax").await, [2]);

    sqlx::query("UPDATE accounts SET type_id = (SELECT id FROM account_types WHERE name = 'daily') WHERE id = 2")
        .execute(&pool)
        .await?;
    assert_eq!(search_accounts(&pool, "Tax daily").await, [2]);
    let duplicates: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM (
           SELECT kind, entity_id FROM search_fts GROUP BY kind, entity_id HAVING COUNT(*) > 1
         )",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(duplicates, 0);
    Ok(())
}

#[tokio::test]
async fn rollback_restores_labels_assignments_and_search() -> anyhow::Result<()> {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "INSERT INTO labels (id, name, name_key) VALUES (1, 'ISA', 'isa');
         INSERT INTO account_labels VALUES (1, 1);",
    )
    .execute(&pool)
    .await?;

    let mut tx = pool.begin().await?;
    sqlx::raw_sql(
        "UPDATE accounts SET name = 'Changed account' WHERE id = 1;
         DELETE FROM labels WHERE id = 1;
         INSERT INTO labels (id, name, name_key) VALUES (2, 'Pending', 'pending');
         INSERT INTO account_labels VALUES (1, 2);",
    )
    .execute(&mut *tx)
    .await?;
    let error = sqlx::query("INSERT INTO account_labels VALUES (1, 999)")
        .execute(&mut *tx)
        .await
        .expect_err("a later assignment fails");
    assert!(
        error
            .as_database_error()
            .unwrap()
            .is_foreign_key_violation()
    );
    tx.rollback().await?;

    assert_eq!(search_accounts(&pool, "ISA Everyday").await, [1]);
    assert!(
        search_accounts(&pool, "Changed OR Pending")
            .await
            .is_empty()
    );
    let labels: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM labels ORDER BY id")
        .fetch_all(&pool)
        .await?;
    assert_eq!(labels, [(1, "ISA".to_string())]);
    let assignments: Vec<(i64, i64)> =
        sqlx::query_as("SELECT account_id, label_id FROM account_labels")
            .fetch_all(&pool)
            .await?;
    assert_eq!(assignments, [(1, 1)]);
    Ok(())
}
