use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::contracts::{AccountIdentifierInput, canonicalize_account_identifier};

pub async fn list(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Vec<super::rows::AccountIdentifierRow>, sqlx::Error> {
    sqlx::query_as("SELECT account_id, position, label, value, search_value FROM account_identifiers WHERE account_id = ? ORDER BY position")
        .bind(account_id)
        .fetch_all(pool)
        .await
}

pub async fn replace_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    account_id: i64,
    identifiers: &[AccountIdentifierInput],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM account_identifiers WHERE account_id = ?")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;
    if identifiers.is_empty() {
        return Ok(());
    }
    let mut insert = QueryBuilder::<Sqlite>::new(
        "INSERT INTO account_identifiers (account_id, position, label, value, search_value) ",
    );
    insert.push_values(
        identifiers.iter().enumerate(),
        |mut row, (position, identifier)| {
            row.push_bind(account_id)
                .push_bind(position as i64)
                .push_bind(&identifier.label)
                .push_bind(&identifier.value)
                .push_bind(canonicalize_account_identifier(&identifier.value));
        },
    );
    insert.build().execute(&mut **tx).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn identifier_schema_allows_empty_values_and_enforces_constraints() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();
        sqlx::migrate!("./db/migrations").run(&pool).await.unwrap();
        sqlx::raw_sql(
        "INSERT INTO institutions (id, name) VALUES (1, 'Test Bank');
         INSERT INTO accounts (id, name, institution_id, type_id, currency_code, account_classification)
         SELECT 1, 'Everyday', 1, id, 'GBP', 'asset' FROM account_types WHERE name = 'current';
         INSERT INTO account_identifiers (account_id, position, label, value, search_value)
         VALUES (1, 0, 'Account number', '0012 3456', '00123456');"
    ).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO account_identifiers (account_id, position, label, value, search_value) VALUES (1, 1, 'IBAN', '', '')")
        .execute(&pool).await.unwrap();
        assert!(
            sqlx::query("UPDATE account_identifiers SET value = ? WHERE position = 1")
                .bind("x".repeat(251))
                .execute(&pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("UPDATE account_identifiers SET label = '' WHERE position = 1")
                .execute(&pool)
                .await
                .is_err()
        );
        sqlx::query("DELETE FROM accounts WHERE id = 1")
            .execute(&pool)
            .await
            .unwrap();
        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_identifiers")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining, 0);
    }
}
