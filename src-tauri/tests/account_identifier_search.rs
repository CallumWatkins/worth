use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use worth_lib::contracts;

#[allow(dead_code)]
#[path = "../src/db/mod.rs"]
mod db;

async fn seeded_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(":memory:")
                .foreign_keys(true),
        )
        .await
        .unwrap();
    sqlx::migrate!("./db/migrations").run(&pool).await.unwrap();
    sqlx::raw_sql(
        "INSERT INTO institutions (id, name) VALUES (1, 'Example Bank');
         INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
         SELECT 1, 'Everyday', 1, id, 'GBP', 'asset' FROM account_types WHERE name = 'current';
         INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
         SELECT 2, 'Reserve', 1, id, 'EUR', 'asset' FROM account_types WHERE name = 'savings';",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn identifier(
    pool: &SqlitePool,
    account_id: i64,
    position: i64,
    value: &str,
    search_value: &str,
) {
    sqlx::query(
        "INSERT INTO account_identifiers (account_id, position, label, value, search_value)
         VALUES (?, ?, 'Reference', ?, ?)",
    )
    .bind(account_id)
    .bind(position)
    .bind(value)
    .bind(search_value)
    .execute(pool)
    .await
    .unwrap();
}

async fn account_hits(pool: &SqlitePool, query: &str) -> Vec<i64> {
    db::search_global(pool, query)
        .await
        .unwrap()
        .into_iter()
        .filter_map(|row| match row {
            db::GlobalSearchRow::Account { id, .. } => Some(id),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn formatted_international_identifiers_preserve_leading_zeroes_and_share_matching() {
    let pool = seeded_pool().await;
    identifier(&pool, 1, 0, "00-12-34", "001234").await;
    identifier(
        &pool,
        1,
        1,
        "GB82 WEST 1234 5698 7654 32",
        "gb82west12345698765432",
    )
    .await;
    identifier(&pool, 2, 0, "CAFÉ-東京", "café東京").await;

    assert_eq!(account_hits(&pool, "00 12 34").await, [1]);
    assert_eq!(account_hits(&pool, "00.12.34").await, [1]);
    assert_eq!(account_hits(&pool, "00\u{2011}12\u{2013}34").await, [1]);
    assert_eq!(
        account_hits(&pool, "gb82 west 1234 5698 7654 32").await,
        [1]
    );
    assert_eq!(account_hits(&pool, "5432").await, [1]);
    assert_eq!(account_hits(&pool, "CAFÉ 東京").await, [2]);
    assert!(account_hits(&pool, "123").await.is_empty());
}

#[tokio::test]
async fn exact_short_identifiers_match_but_partial_threshold_counts_characters() {
    let pool = seeded_pool().await;
    identifier(&pool, 1, 0, "007", "007").await;
    identifier(&pool, 1, 1, "東京駅中央", "東京駅中央").await;
    identifier(&pool, 2, 0, "東京駅", "東京駅").await;

    assert_eq!(account_hits(&pool, "007").await, [1]);
    assert!(account_hits(&pool, "07").await.is_empty());
    assert_eq!(account_hits(&pool, "東京駅").await, [2]);
    assert_eq!(account_hits(&pool, "東京駅中").await, [1]);
}

#[tokio::test]
async fn punctuation_is_literal_and_empty_canonical_queries_do_not_match_everything() {
    let pool = seeded_pool().await;
    identifier(&pool, 1, 0, "ref%_123", "ref%_123").await;
    identifier(&pool, 1, 1, "////", "////").await;
    identifier(&pool, 2, 0, "refab123", "refab123").await;
    identifier(&pool, 2, 1, "ABC/123", "abc/123").await;

    assert_eq!(account_hits(&pool, "ref%_").await, [1]);
    assert_eq!(account_hits(&pool, "////").await, [1]);
    assert_eq!(account_hits(&pool, "ABC/123").await, [2]);
    assert!(account_hits(&pool, "abc123").await.is_empty());
    assert!(account_hits(&pool, "").await.is_empty());
    assert!(account_hits(&pool, " \t- . —").await.is_empty());
    assert!(account_hits(&pool, "\" OR *").await.is_empty());
    assert!(account_hits(&pool, "NOT").await.is_empty());
    assert!(account_hits(&pool, "AND").await.is_empty());
    assert!(account_hits(&pool, "ref OR everyday").await.is_empty());
}

#[tokio::test]
async fn fts_operator_words_are_searched_as_literal_text() {
    let pool = seeded_pool().await;
    sqlx::query("UPDATE accounts SET name = 'AND OR NOT' WHERE id = 2")
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(account_hits(&pool, "AND").await, [2]);
    assert_eq!(account_hits(&pool, "OR").await, [2]);
    assert_eq!(account_hits(&pool, "NOT").await, [2]);
    assert_eq!(account_hits(&pool, "AND OR NOT").await, [2]);
    assert!(account_hits(&pool, "everyday OR missing").await.is_empty());
}

#[tokio::test]
async fn shared_identifiers_and_multiple_matches_return_each_account_once() {
    let pool = seeded_pool().await;
    identifier(&pool, 1, 0, "Everyday", "everyday").await;
    identifier(&pool, 1, 1, "EVERYDAY", "everyday").await;
    identifier(&pool, 2, 0, "Everyday", "everyday").await;

    assert_eq!(account_hits(&pool, "everyday").await, [1, 2]);
    assert_eq!(account_hits(&pool, "eryd").await, [1, 2]);
}

#[tokio::test]
async fn exact_matches_precede_text_then_partial_matches_before_the_final_limit() {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "WITH RECURSIVE numbers(id) AS (VALUES(3) UNION ALL SELECT id + 1 FROM numbers WHERE id < 15)
         INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
         SELECT numbers.id, 'Account ' || numbers.id, 1, account_types.id, 'GBP', 'asset'
         FROM numbers CROSS JOIN account_types WHERE account_types.name = 'current';
         INSERT INTO account_identifiers (account_id, position, label, value, search_value)
         SELECT id, 0, 'Reference', 'XX1234YY', 'xx1234yy' FROM accounts;
         INSERT INTO account_identifiers (account_id, position, label, value, search_value)
         VALUES (15, 1, 'Number', '1234', '1234');
         UPDATE accounts SET name = '1234' WHERE id = 14;",
    )
    .execute(&pool)
    .await
    .unwrap();

    let hits = account_hits(&pool, "1234").await;
    assert_eq!(hits.len(), 10);
    assert_eq!(&hits[..2], [15, 14]);
    let mut unique_hits = hits.clone();
    unique_hits.sort_unstable();
    unique_hits.dedup();
    assert_eq!(unique_hits.len(), hits.len());
}

#[tokio::test]
async fn text_search_keeps_institution_label_and_account_hits() {
    let pool = seeded_pool().await;
    sqlx::raw_sql(
        "INSERT INTO labels (id, name, name_key) VALUES (1, 'Travel fund', 'travel fund');
         INSERT INTO account_labels (account_id, label_id) VALUES (2, 1);",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(account_hits(&pool, "every").await, [1]);
    assert_eq!(account_hits(&pool, "trav fun").await, [2]);
    assert!(
        db::search_global(&pool, "trav fun")
            .await
            .unwrap()
            .iter()
            .any(|row| matches!(
                row,
                db::GlobalSearchRow::Label {
                    id: 1,
                    account_count: 1,
                    ..
                }
            ))
    );
    assert!(
        db::search_global(&pool, "example")
            .await
            .unwrap()
            .iter()
            .any(|row| matches!(row, db::GlobalSearchRow::Institution { id: 1, .. }))
    );
}

#[tokio::test]
async fn edits_clears_and_cascading_deletion_remove_old_matches() {
    let pool = seeded_pool().await;
    identifier(&pool, 1, 0, "001122", "001122").await;
    identifier(&pool, 2, 0, "001122", "001122").await;
    sqlx::query("UPDATE account_identifiers SET value = '334455', search_value = '334455' WHERE account_id = 1")
        .execute(&pool).await.unwrap();
    assert_eq!(account_hits(&pool, "001122").await, [2]);
    assert_eq!(account_hits(&pool, "334455").await, [1]);

    sqlx::query("DELETE FROM account_identifiers WHERE account_id = 1")
        .execute(&pool)
        .await
        .unwrap();
    assert!(account_hits(&pool, "334455").await.is_empty());
    sqlx::query("DELETE FROM institutions WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    assert!(account_hits(&pool, "001122").await.is_empty());
}
