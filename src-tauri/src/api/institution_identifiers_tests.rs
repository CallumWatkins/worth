use super::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

async fn test_pool() -> SqlitePool {
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
    pool
}

fn identifier(label: &str, value: &str) -> IdentifierInput {
    IdentifierInput {
        label: label.to_owned(),
        value: value.to_owned(),
    }
}

fn input() -> InstitutionUpsertInput {
    InstitutionUpsertInput {
        name: "Test Bank".to_owned(),
        identifiers: vec![
            identifier("Customer number", "0012 3456"),
            identifier("Reference", "ABC/001"),
        ],
    }
}

#[tokio::test]
async fn identifiers_round_trip_without_accounts_and_stay_out_of_summaries() {
    let pool = test_pool().await;
    let mut input = input();
    input.identifiers.insert(0, identifier(" ", " "));
    input
        .identifiers
        .push(identifier(" Customer number ", " 0099-88 "));
    input.identifiers.push(identifier("Membership number", ""));
    let id = save_institution(&pool, &input, None).await.unwrap();
    let detail = institution_detail_by_id(&pool, id).await.unwrap();
    assert!(detail.accounts.is_empty());
    assert_eq!(
        detail.identifiers,
        [
            identifier("Customer number", "0012 3456"),
            identifier("Reference", "ABC/001"),
            identifier("Customer number", "0099-88"),
            identifier("Membership number", "")
        ]
    );
    assert!(
        !serde_json::to_string(&detail)
            .unwrap()
            .contains("search_value")
    );
    let summaries = institution_summaries(&pool).await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].id, id);
    let serialized = serde_json::to_string(&summaries).unwrap();
    assert!(!serialized.contains("identifiers"));
    assert!(!serialized.contains("0012"));
    let search = search_results(&pool, "009988").await.unwrap();
    assert!(
        matches!(search.as_slice(), [SearchResultDto::Institution { id: found, .. }] if *found == id)
    );
    let serialized = serde_json::to_string(&search).unwrap();
    assert!(!serialized.contains("identifiers"));
    assert!(!serialized.contains("0099"));
}

#[tokio::test]
async fn reorder_edit_and_clear_replace_identifiers_and_search_values() {
    let pool = test_pool().await;
    let mut input = input();
    let id = save_institution(&pool, &input, None).await.unwrap();
    input.identifiers.reverse();
    input.identifiers[1] = identifier("New reference", "NEW/1234");
    save_institution(&pool, &input, Some(id)).await.unwrap();
    assert_eq!(
        institution_detail_by_id(&pool, id)
            .await
            .unwrap()
            .identifiers,
        input.identifiers
    );
    assert!(
        db::search_global(&pool, "00123456")
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(db::search_global(&pool, "new/1234").await.unwrap().len(), 1);
    input.identifiers = vec![identifier("", ""), identifier(" ", " ")];
    save_institution(&pool, &input, Some(id)).await.unwrap();
    assert!(
        institution_detail_by_id(&pool, id)
            .await
            .unwrap()
            .identifiers
            .is_empty()
    );
    assert!(
        db::search_global(&pool, "new/1234")
            .await
            .unwrap()
            .is_empty()
    );
    input.name = "Missing bank".to_owned();
    assert!(matches!(
        save_institution(&pool, &input, Some(id + 1)).await,
        Err(ApiError::NotFound)
    ));
}

#[tokio::test]
async fn invalid_identifiers_have_indexed_safe_errors_and_leave_existing_details_intact() {
    let pool = test_pool().await;
    let original = input();
    let id = save_institution(&pool, &original, None).await.unwrap();
    for (bad, field) in [
        (identifier("", "secret"), "identifiers.0.label"),
        (identifier("Reference", "sec\nret"), "identifiers.0.value"),
    ] {
        let invalid = InstitutionUpsertInput {
            name: "Renamed".to_owned(),
            identifiers: vec![bad],
        };
        for target in [None, Some(id)] {
            let Err(ApiError::Validation(issues)) = save_institution(&pool, &invalid, target).await
            else {
                panic!("expected validation error");
            };
            assert_eq!(issues[0].field, field);
            assert!(!serde_json::to_string(&issues).unwrap().contains("secret"));
        }
    }
    let detail = institution_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(detail.name, original.name);
    assert_eq!(detail.identifiers, original.identifiers);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM institutions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn failed_identifier_writes_roll_back_creation_name_changes_and_replacement() {
    let pool = test_pool().await;
    let original = input();
    let id = save_institution(&pool, &original, None).await.unwrap();
    sqlx::raw_sql("CREATE TRIGGER reject_identifier BEFORE INSERT ON institution_identifiers BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        .execute(&pool).await.unwrap();
    let changed = InstitutionUpsertInput {
        name: "Another bank".to_owned(),
        identifiers: vec![identifier("New", "987654")],
    };
    for target in [Some(id), None] {
        assert!(matches!(
            save_institution(&pool, &changed, target).await,
            Err(ApiError::Db)
        ));
    }
    let detail = institution_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(detail.name, original.name);
    assert_eq!(detail.identifiers, original.identifiers);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM institutions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}

fn account_input() -> AccountUpsertInput {
    AccountUpsertInput {
        institution: InstitutionRef::New { input: input() },
        name: "Everyday".to_owned(),
        account_type: AccountTypeName::Current,
        currency_code: CurrencyCode::GBP,
        account_classification: AccountClassification::Asset,
        include_in_dashboard: true,
        labels: Vec::new(),
        identifiers: vec![identifier("Account number", "ACCOUNT/001")],
        opened_date: None,
        closed_date: None,
    }
}

#[tokio::test]
async fn inline_creation_preserves_ownership_through_account_updates_moves_and_deletion() {
    let pool = test_pool().await;
    let mut account = account_input();
    let id = save_account(&pool, &account, None).await.unwrap();
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    let institution_id = detail.account.institution.id;
    assert_eq!(detail.identifiers, account.identifiers);
    assert_eq!(
        institution_detail_by_id(&pool, institution_id)
            .await
            .unwrap()
            .identifiers,
        input().identifiers
    );
    assert!(
        matches!(db::search_global(&pool, "00123456").await.unwrap().as_slice(), [db::GlobalSearchRow::Institution { id, .. }] if *id == institution_id)
    );
    sqlx::query("INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor) VALUES (?, '2026-01-01', 12345)")
        .bind(id).execute(&pool).await.unwrap();
    account.institution = InstitutionRef::Existing { id: institution_id };
    account.name = "Updated account".to_owned();
    save_account(&pool, &account, Some(id)).await.unwrap();
    let mut institution = input();
    institution.identifiers.reverse();
    save_institution(&pool, &institution, Some(institution_id))
        .await
        .unwrap();
    assert_eq!(
        account_detail_by_id(&pool, id)
            .await
            .unwrap()
            .account
            .latest_balance_minor,
        12345
    );
    let other_id = save_institution(
        &pool,
        &InstitutionUpsertInput {
            name: "Other bank".to_owned(),
            identifiers: Vec::new(),
        },
        None,
    )
    .await
    .unwrap();
    account.institution = InstitutionRef::Existing { id: other_id };
    save_account(&pool, &account, Some(id)).await.unwrap();
    assert_eq!(
        institution_detail_by_id(&pool, institution_id)
            .await
            .unwrap()
            .identifiers,
        institution.identifiers
    );
    assert!(
        institution_detail_by_id(&pool, other_id)
            .await
            .unwrap()
            .identifiers
            .is_empty()
    );
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        db::institution_identifiers::list(&pool, institution_id)
            .await
            .unwrap()
            .len(),
        2
    );
    sqlx::query("DELETE FROM institutions WHERE id = ?")
        .bind(institution_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        db::institution_identifiers::list(&pool, institution_id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        db::search_global(&pool, "00123456")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn inline_creation_validates_nested_identifiers_and_rolls_back_with_account_failure() {
    let pool = test_pool().await;
    let mut account = account_input();
    if let InstitutionRef::New { input } = &mut account.institution {
        input.identifiers = vec![identifier("", "secret")];
    }
    let Err(ApiError::Validation(issues)) = save_account(&pool, &account, None).await else {
        panic!("expected nested validation error");
    };
    assert_eq!(issues[0].field, "institution.input.identifiers.0.label");
    sqlx::raw_sql("CREATE TRIGGER reject_account_identifier BEFORE INSERT ON account_identifiers BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        .execute(&pool).await.unwrap();
    assert!(matches!(
        save_account(&pool, &account_input(), None).await,
        Err(ApiError::Db)
    ));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM institutions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM institution_identifiers")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}
