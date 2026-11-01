use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::contracts::{IdentifierInput, canonicalize_identifier};

pub async fn list(
    pool: &SqlitePool,
    institution_id: i64,
) -> Result<Vec<super::rows::InstitutionIdentifierRow>, sqlx::Error> {
    sqlx::query_as("SELECT institution_id, position, label, value, search_value FROM institution_identifiers WHERE institution_id = ? ORDER BY position")
        .bind(institution_id)
        .fetch_all(pool)
        .await
}

pub async fn replace_tx(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    institution_id: i64,
    identifiers: &[IdentifierInput],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM institution_identifiers WHERE institution_id = ?")
        .bind(institution_id)
        .execute(&mut **tx)
        .await?;
    if identifiers.is_empty() {
        return Ok(());
    }
    let mut insert = QueryBuilder::<Sqlite>::new(
        "INSERT INTO institution_identifiers (institution_id, position, label, value, search_value) ",
    );
    insert.push_values(
        identifiers.iter().enumerate(),
        |mut row, (position, identifier)| {
            row.push_bind(institution_id)
                .push_bind(position as i64)
                .push_bind(&identifier.label)
                .push_bind(&identifier.value)
                .push_bind(canonicalize_identifier(&identifier.value));
        },
    );
    insert.build().execute(&mut **tx).await?;
    Ok(())
}
