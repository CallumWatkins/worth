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

fn identifier(label: &str, value: &str) -> AccountIdentifierInput {
    AccountIdentifierInput {
        label: label.to_owned(),
        value: value.to_owned(),
    }
}

fn input() -> AccountUpsertInput {
    AccountUpsertInput {
        institution: InstitutionRef::New {
            input: InstitutionUpsertInput {
                name: "Test Bank".to_owned(),
            },
        },
        name: "Everyday".to_owned(),
        account_type: AccountTypeName::Current,
        currency_code: CurrencyCode::GBP,
        account_classification: AccountClassification::Asset,
        include_in_dashboard: true,
        labels: Vec::new(),
        identifiers: vec![
            identifier("Account number", "0012 3456"),
            identifier("銀行コード", "0001"),
            identifier("Account number", "0099/88"),
        ],
        opened_date: None,
        closed_date: None,
    }
}

#[tokio::test]
async fn identifiers_round_trip_in_order_without_leaking_into_lists_or_search_results() {
    let pool = test_pool().await;
    let mut input = input();
    input.identifiers[0].label = "  Account number  ".to_owned();
    let id = save_account(&pool, &input, None).await.unwrap();
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(
        detail.identifiers[0],
        identifier("Account number", "0012 3456")
    );
    assert_eq!(detail.identifiers[1..], input.identifiers[1..]);
    let serialized = serde_json::to_value(&detail).unwrap();
    assert_eq!(serialized["id"], id);
    assert!(serialized.get("account").is_none());
    assert!(serialized.get("search_value").is_none());
    let list = build_account_dtos(&pool, db::accounts_list_full(&pool).await.unwrap())
        .await
        .unwrap();
    let serialized_list = serde_json::to_string(&list).unwrap();
    assert!(!serialized_list.contains("0012"));
    assert!(!serialized_list.contains("identifiers"));
    let search = db::search_global(&pool, "00123456").await.unwrap();
    assert!(
        matches!(search.as_slice(), [db::GlobalSearchRow::Account { id: found, .. }] if *found == id)
    );
    let stored = db::account_identifiers::list(&pool, id).await.unwrap();
    assert_eq!(stored[0].search_value, "00123456");
}

#[tokio::test]
async fn changing_reordering_and_clearing_details_updates_search_and_preserves_balances() {
    let pool = test_pool().await;
    let mut input = input();
    let id = save_account(&pool, &input, None).await.unwrap();
    input.institution = InstitutionRef::Existing {
        id: account_detail_by_id(&pool, id)
            .await
            .unwrap()
            .account
            .institution
            .id,
    };
    sqlx::query("INSERT INTO account_balance_snapshots (account_id, balance_date, balance_minor) VALUES (?, '2026-01-01', 12345)")
        .bind(id).execute(&pool).await.unwrap();
    input.identifiers.reverse();
    input.identifiers[2] = identifier("Policy number", "NEW/1234");
    save_account(&pool, &input, Some(id)).await.unwrap();
    assert_eq!(
        account_detail_by_id(&pool, id).await.unwrap().identifiers,
        input.identifiers
    );
    assert!(
        db::search_global(&pool, "00123456")
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(db::search_global(&pool, "new/1234").await.unwrap().len(), 1);
    input.identifiers.clear();
    save_account(&pool, &input, Some(id)).await.unwrap();
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    assert!(detail.identifiers.is_empty());
    assert_eq!(detail.account.latest_balance_minor, 12345);
    assert!(
        db::search_global(&pool, "new/1234")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn invalid_details_have_indexed_safe_errors_and_do_not_create_accounts() {
    let pool = test_pool().await;
    for (detail, field) in [
        (identifier(" ", "00123456"), "identifiers.0.label"),
        (identifier("Number", "12\t34"), "identifiers.0.value"),
        (identifier("Number", "123\n456"), "identifiers.0.value"),
        (identifier("Number", "123\0secret"), "identifiers.0.value"),
        (identifier(&"名".repeat(81), "123"), "identifiers.0.label"),
        (
            identifier("Number", &"💷".repeat(251)),
            "identifiers.0.value",
        ),
    ] {
        let mut input = input();
        input.identifiers = vec![detail];
        let Err(ApiError::Validation(issues)) = save_account(&pool, &input, None).await else {
            panic!("expected validation error");
        };
        assert_eq!(issues[0].field, field);
        assert!(!serde_json::to_string(&issues).unwrap().contains("secret"));
    }
    let mut input = input();
    assert!(db::accounts_list_full(&pool).await.unwrap().is_empty());
    input.identifiers = vec![identifier(&"名".repeat(80), &"💷".repeat(250)); 200];
    let id = save_account(&pool, &input, None).await.unwrap();
    assert_eq!(
        account_detail_by_id(&pool, id)
            .await
            .unwrap()
            .identifiers
            .len(),
        200
    );
}

#[tokio::test]
async fn saving_discards_fully_blank_identifiers_but_keeps_meaningful_rows() {
    let pool = test_pool().await;
    let mut input = input();
    input.identifiers = vec![
        identifier("", ""),
        identifier(" IBAN ", ""),
        identifier(" \t", "\u{00a0}\u{3000}"),
        identifier("Reference", " 0012 3456 "),
        identifier("", " "),
    ];
    let id = save_account(&pool, &input, None).await.unwrap();
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(
        detail.identifiers,
        [identifier("IBAN", ""), identifier("Reference", "0012 3456")]
    );
    let rows = db::account_identifiers::list(&pool, id).await.unwrap();
    assert_eq!(
        rows.iter().map(|row| row.position).collect::<Vec<_>>(),
        [0, 1]
    );

    input.institution = InstitutionRef::Existing {
        id: detail.account.institution.id,
    };
    input.identifiers = vec![identifier("", ""), identifier(" ", "123456")];
    assert!(matches!(
        save_account(&pool, &input, Some(id)).await,
        Err(ApiError::Validation(_))
    ));
    assert_eq!(
        account_detail_by_id(&pool, id).await.unwrap().identifiers,
        detail.identifiers
    );

    input.identifiers = vec![identifier("", ""), identifier("\t ", "\u{00a0}")];
    save_account(&pool, &input, Some(id)).await.unwrap();
    assert!(
        account_detail_by_id(&pool, id)
            .await
            .unwrap()
            .identifiers
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
async fn empty_identifier_values_round_trip_and_can_be_filled_or_cleared() {
    let pool = test_pool().await;
    let mut input = input();
    input.identifiers = vec![identifier("IBAN", ""), identifier("Reference", "  ")];
    let id = save_account(&pool, &input, None).await.unwrap();
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(
        detail.identifiers,
        [identifier("IBAN", ""), identifier("Reference", "")]
    );
    let stored = db::account_identifiers::list(&pool, id).await.unwrap();
    assert!(
        stored
            .iter()
            .all(|row| row.value.is_empty() && row.search_value.is_empty())
    );
    assert!(db::search_global(&pool, "").await.unwrap().is_empty());
    assert!(db::search_global(&pool, " - . ").await.unwrap().is_empty());

    input.institution = InstitutionRef::Existing {
        id: detail.account.institution.id,
    };
    input.identifiers[0].value = "0012 3456".to_owned();
    save_account(&pool, &input, Some(id)).await.unwrap();
    assert_eq!(db::search_global(&pool, "00123456").await.unwrap().len(), 1);
    input.identifiers[0].value.clear();
    save_account(&pool, &input, Some(id)).await.unwrap();
    assert_eq!(
        account_detail_by_id(&pool, id).await.unwrap().identifiers[0],
        identifier("IBAN", "")
    );
    assert!(
        db::search_global(&pool, "00123456")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn identifier_write_failures_roll_back_the_entire_account_save() {
    let pool = test_pool().await;
    let mut input = input();
    let id = save_account(&pool, &input, None).await.unwrap();
    let original = account_detail_by_id(&pool, id).await.unwrap();
    sqlx::raw_sql("CREATE TRIGGER reject_identifier BEFORE INSERT ON account_identifiers BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        .execute(&pool).await.unwrap();
    input.name = "Renamed".to_owned();
    input.institution = InstitutionRef::New {
        input: InstitutionUpsertInput {
            name: "Another bank".to_owned(),
        },
    };
    input.identifiers = vec![identifier("New", "987654")];
    assert!(matches!(
        save_account(&pool, &input, Some(id)).await,
        Err(ApiError::Db)
    ));
    let detail = account_detail_by_id(&pool, id).await.unwrap();
    assert_eq!(detail.identifiers, original.identifiers);
    assert_eq!(detail.account.name, original.account.name);
    assert_eq!(
        detail.account.institution.id,
        original.account.institution.id
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM institutions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn deleting_accounts_or_institutions_cascades_to_identifiers() {
    let pool = test_pool().await;
    for delete_institution in [false, true] {
        let id = save_account(&pool, &input(), None).await.unwrap();
        let institution_id = account_detail_by_id(&pool, id)
            .await
            .unwrap()
            .account
            .institution
            .id;
        if delete_institution {
            sqlx::query("DELETE FROM institutions WHERE id = ?")
                .bind(institution_id)
                .execute(&pool)
                .await
                .unwrap();
        } else {
            sqlx::query("DELETE FROM accounts WHERE id = ?")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM institutions WHERE id = ?")
                .bind(institution_id)
                .execute(&pool)
                .await
                .unwrap();
        }
        assert!(
            db::account_identifiers::list(&pool, id)
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
        assert!(matches!(
            account_detail_by_id(&pool, id).await,
            Err(ApiError::NotFound)
        ));
    }
}
