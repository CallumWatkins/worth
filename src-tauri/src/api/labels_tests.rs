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

fn label_input(name: &str) -> LabelUpsertInput {
    LabelUpsertInput {
        name: name.to_owned(),
    }
}

fn new_label(name: &str) -> LabelRef {
    LabelRef::New {
        input: label_input(name),
    }
}

fn account_input(
    name: &str,
    institution: InstitutionRef,
    labels: Vec<LabelRef>,
) -> AccountUpsertInput {
    AccountUpsertInput {
        institution,
        name: name.to_owned(),
        account_type: AccountTypeName::Savings,
        currency_code: CurrencyCode::GBP,
        account_classification: AccountClassification::Asset,
        include_in_dashboard: true,
        opened_date: None,
        closed_date: None,
        labels,
    }
}

fn new_institution() -> InstitutionRef {
    InstitutionRef::New {
        input: InstitutionUpsertInput {
            name: "Test Bank".to_owned(),
        },
    }
}

fn assert_issue<T>(result: Result<T, ApiError>, field: &str, message: &str) {
    let Err(ApiError::Validation(issues)) = result else {
        panic!("expected validation error");
    };
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].field, field);
    assert_eq!(issues[0].message, message);
}

#[tokio::test]
async fn label_management_normalizes_names_and_rejects_duplicate_renames() {
    let pool = test_pool().await;
    let isa = save_label(&pool, &label_input(" \u{2003}ISA\t "), None)
        .await
        .unwrap();
    let labels = db::labels_list(&pool).await.unwrap();
    assert_eq!(labels[0].name, "ISA");
    assert_eq!(labels[0].account_count, 0);
    assert_issue(
        save_label(&pool, &label_input("isa"), None).await,
        "name",
        "A label with this name already exists",
    );
    assert_eq!(
        save_label(&pool, &label_input("Isa"), Some(isa))
            .await
            .unwrap(),
        isa
    );
    let other = save_label(&pool, &label_input("House deposit"), None)
        .await
        .unwrap();
    assert_issue(
        save_label(&pool, &label_input(" ISA "), Some(other)).await,
        "name",
        "A label with this name already exists",
    );
    let labels = db::labels_list(&pool).await.unwrap();
    assert_eq!(
        labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
        ["House deposit", "Isa"]
    );

    save_label(&pool, &label_input("ÉPARGNE"), None)
        .await
        .unwrap();
    assert_issue(
        save_label(&pool, &label_input(" épargne "), None).await,
        "name",
        "A label with this name already exists",
    );
    assert!(matches!(
        save_label(&pool, &label_input("Missing"), Some(999)).await,
        Err(ApiError::NotFound)
    ));
    assert!(db::label_delete(&pool, other).await.unwrap());
    assert!(!db::label_delete(&pool, other).await.unwrap());
}

#[tokio::test]
async fn label_validation_counts_unicode_characters_after_trimming() {
    let pool = test_pool().await;
    save_label(&pool, &label_input("A"), None).await.unwrap();
    save_label(&pool, &label_input(&format!(" {} ", "💷".repeat(20))), None)
        .await
        .unwrap();
    assert_issue(
        save_label(&pool, &label_input(&"💷".repeat(21)), None).await,
        "name",
        "Label name must be 20 characters or fewer",
    );
    assert_issue(
        save_label(&pool, &label_input(" \t\u{2003} "), None).await,
        "name",
        "Enter a label name",
    );
    assert_issue(
        save_label(&pool, &label_input("Hidden\0suffix"), None).await,
        "name",
        "Enter a label name",
    );

    let invalid_name = account_input(
        "Savings",
        new_institution(),
        vec![new_label(&"x".repeat(21))],
    );
    assert_issue(
        save_account(&pool, &invalid_name, None).await,
        "labels.0.input.name",
        "Label name must be 20 characters or fewer",
    );
    let invalid_id = account_input(
        "Savings",
        new_institution(),
        vec![LabelRef::Existing { id: 0 }],
    );
    assert_issue(
        save_account(&pool, &invalid_id, None).await,
        "labels.0.id",
        "Select a label",
    );
}

#[tokio::test]
async fn accounts_share_labels_and_return_sorted_assignments_without_duplicates() {
    let pool = test_pool().await;
    let isa = save_label(&pool, &label_input("ISA"), None).await.unwrap();
    save_label(&pool, &label_input("Unused"), None)
        .await
        .unwrap();
    let first = save_account(
        &pool,
        &account_input(
            "Savings",
            new_institution(),
            vec![
                LabelRef::Existing { id: isa },
                new_label(" isa "),
                new_label("House deposit"),
                new_label("HOUSE DEPOSIT"),
            ],
        ),
        None,
    )
    .await
    .unwrap();
    let account = account_dto_by_id(&pool, first).await.unwrap();
    assert_eq!(
        account
            .labels
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["House deposit", "ISA"]
    );
    let institution_id = account.institution.id;
    let second_input = account_input(
        "Investments",
        InstitutionRef::Existing { id: institution_id },
        vec![new_label("ISA")],
    );
    let second = save_account(&pool, &second_input, None).await.unwrap();
    let labels = db::labels_list(&pool).await.unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|l| (l.name.as_str(), l.account_count))
            .collect::<Vec<_>>(),
        [("House deposit", 1), ("ISA", 2), ("Unused", 0)]
    );

    save_label(&pool, &label_input("UK ISA"), Some(isa))
        .await
        .unwrap();
    let institution = institution_detail_by_id(&pool, institution_id)
        .await
        .unwrap();
    assert!(
        institution
            .accounts
            .iter()
            .all(|a| a.labels.iter().any(|l| l.id == isa && l.name == "UK ISA"))
    );
    let results = db::search_global(&pool, "UK ISA").await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(
        |r| matches!(r, db::GlobalSearchRow::Account { id, .. } if *id == first || *id == second)
    ));

    let mut updated = second_input;
    updated.labels.clear();
    save_account(&pool, &updated, Some(second)).await.unwrap();
    assert!(
        account_dto_by_id(&pool, second)
            .await
            .unwrap()
            .labels
            .is_empty()
    );
    assert_eq!(
        db::labels_list(&pool)
            .await
            .unwrap()
            .iter()
            .find(|l| l.id == isa)
            .unwrap()
            .account_count,
        1
    );
    assert!(db::label_delete(&pool, isa).await.unwrap());
    assert_eq!(
        account_dto_by_id(&pool, first).await.unwrap().labels.len(),
        1
    );
    assert!(db::search_global(&pool, "UK ISA").await.unwrap().is_empty());
}

#[tokio::test]
async fn failed_account_creation_rolls_back_new_institution_and_new_labels() {
    let pool = test_pool().await;
    let input = account_input(
        "Savings",
        new_institution(),
        vec![new_label("Pending"), LabelRef::Existing { id: 999 }],
    );
    assert_issue(
        save_account(&pool, &input, None).await,
        "labels.1.id",
        "Label does not exist",
    );
    let counts: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM accounts), (SELECT COUNT(*) FROM institutions),
                (SELECT COUNT(*) FROM labels), (SELECT COUNT(*) FROM account_labels),
                (SELECT COUNT(*) FROM search_fts)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (0, 0, 0, 0, 0));
}

#[tokio::test]
async fn failed_account_update_preserves_account_labels_and_search() {
    let pool = test_pool().await;
    let mut input = account_input("Savings", new_institution(), vec![new_label("ISA")]);
    let id = save_account(&pool, &input, None).await.unwrap();
    let account = account_dto_by_id(&pool, id).await.unwrap();
    input.name = "Changed".to_owned();
    input.institution = InstitutionRef::New {
        input: InstitutionUpsertInput {
            name: "Pending bank".to_owned(),
        },
    };
    input.labels = vec![new_label("Pending"), LabelRef::Existing { id: 999 }];
    assert_issue(
        save_account(&pool, &input, Some(id)).await,
        "labels.1.id",
        "Label does not exist",
    );
    let after = account_dto_by_id(&pool, id).await.unwrap();
    assert_eq!(after.name, account.name);
    assert_eq!(after.institution.id, account.institution.id);
    assert_eq!(after.labels[0].id, account.labels[0].id);
    assert_eq!(db::labels_list(&pool).await.unwrap().len(), 1);
    assert!(
        !db::institution_name_exists(&pool, "Pending bank", None)
            .await
            .unwrap()
    );
    assert_eq!(
        db::search_global(&pool, "ISA Savings").await.unwrap().len(),
        1
    );
    assert!(
        db::search_global(&pool, "Pending")
            .await
            .unwrap()
            .is_empty()
    );

    input.institution = InstitutionRef::Existing {
        id: account.institution.id,
    };
    assert_issue(
        save_account(&pool, &input, Some(id)).await,
        "labels.1.id",
        "Label does not exist",
    );
    assert_eq!(account_dto_by_id(&pool, id).await.unwrap().name, "Savings");
    assert!(matches!(
        save_account(&pool, &input, Some(999)).await,
        Err(ApiError::NotFound)
    ));
}

#[tokio::test]
async fn account_label_saves_only_write_changed_assignments() {
    let pool = test_pool().await;
    let isa = save_label(&pool, &label_input("ISA"), None).await.unwrap();
    let house = save_label(&pool, &label_input("House deposit"), None)
        .await
        .unwrap();
    let retirement = save_label(&pool, &label_input("Retirement"), None)
        .await
        .unwrap();
    let mut input = account_input(
        "Savings",
        new_institution(),
        vec![
            LabelRef::Existing { id: isa },
            LabelRef::Existing { id: house },
        ],
    );
    let account_id = save_account(&pool, &input, None).await.unwrap();
    input.institution = InstitutionRef::Existing {
        id: account_dto_by_id(&pool, account_id)
            .await
            .unwrap()
            .institution
            .id,
    };
    sqlx::raw_sql(
        "CREATE TABLE assignment_events (operation TEXT, label_id INTEGER);
         CREATE TRIGGER record_assignment_insert AFTER INSERT ON account_labels BEGIN
           INSERT INTO assignment_events VALUES ('insert', new.label_id);
         END;
         CREATE TRIGGER record_assignment_delete AFTER DELETE ON account_labels BEGIN
           INSERT INTO assignment_events VALUES ('delete', old.label_id);
         END;",
    )
    .execute(&pool)
    .await
    .unwrap();

    input.labels = vec![
        LabelRef::Existing { id: isa },
        LabelRef::Existing { id: retirement },
    ];
    save_account(&pool, &input, Some(account_id)).await.unwrap();
    let events: Vec<(String, i64)> = sqlx::query_as(
        "SELECT operation, label_id FROM assignment_events ORDER BY operation, label_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        events,
        [
            ("delete".to_owned(), house),
            ("insert".to_owned(), retirement)
        ]
    );

    sqlx::query("DELETE FROM assignment_events")
        .execute(&pool)
        .await
        .unwrap();
    save_account(&pool, &input, Some(account_id)).await.unwrap();
    let unchanged_writes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assignment_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(unchanged_writes, 0);

    input.labels.clear();
    save_account(&pool, &input, Some(account_id)).await.unwrap();
    let events: Vec<(String, i64)> = sqlx::query_as(
        "SELECT operation, label_id FROM assignment_events ORDER BY operation, label_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        events,
        [
            ("delete".to_owned(), isa),
            ("delete".to_owned(), retirement)
        ]
    );
    assert!(
        account_dto_by_id(&pool, account_id)
            .await
            .unwrap()
            .labels
            .is_empty()
    );
    assert_eq!(db::labels_list(&pool).await.unwrap().len(), 3);
}

#[tokio::test]
async fn assignment_failure_rolls_back_the_entire_account_save() {
    let pool = test_pool().await;
    let id = save_account(
        &pool,
        &account_input("Savings", new_institution(), vec![new_label("ISA")]),
        None,
    )
    .await
    .unwrap();
    let original = account_dto_by_id(&pool, id).await.unwrap();
    // Exercise rollback after replacing existing assignments has begun, not just validation.
    sqlx::raw_sql("CREATE TRIGGER fail_assignment BEFORE INSERT ON account_labels BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
        .execute(&pool).await.unwrap();
    let changed = account_input(
        "Changed",
        InstitutionRef::Existing {
            id: original.institution.id,
        },
        vec![new_label("Pending")],
    );
    assert!(matches!(
        save_account(&pool, &changed, Some(id)).await,
        Err(ApiError::Db)
    ));
    let after = account_dto_by_id(&pool, id).await.unwrap();
    assert_eq!(after.name, original.name);
    assert_eq!(after.labels[0].id, original.labels[0].id);
    assert_eq!(db::labels_list(&pool).await.unwrap().len(), 1);
    assert_eq!(
        db::search_global(&pool, "ISA Savings").await.unwrap().len(),
        1
    );
}
