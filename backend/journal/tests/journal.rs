use errors::AppError;
use sqlx::PgPool;
use user::GoogleAccount;
use journal::entries::ListOrder;

fn account() -> GoogleAccount {
    GoogleAccount {
        google_account_id: "g-journal".into(),
        name: "Journalist".into(),
        email: Some("journal@example.com".into()),
        email_verified: true,
        avatar_url: None,
    }
}

#[sqlx::test(migrations = "../db/migrations")]
async fn journal_crud_and_journal_limit(pool: PgPool) {
    let user = user::login_with_google(&pool, &account())
        .await
        .expect("user");

    let created =
        journal::journals::create(&pool, user.id, "Notes", b"salt", b"dek-bytes", Some("hint"))
            .await
            .expect("journal");
    assert_eq!(created.name, "Notes");
    assert_eq!(created.passphrase_hint.as_deref(), Some("hint"));
    assert!(!created.mask);

    let notebook =
        journal::notebooks::create(&pool, user.id, created.id, b"notebook-name", Some("book"))
            .await
            .expect("notebook");
    assert_eq!(notebook.name, b"notebook-name");

    let entry = journal::entries::create(&pool, user.id, notebook.id, b"title", b"body", None)
        .await
        .expect("entry");
    assert_eq!(entry.title, b"title");
    assert_eq!(entry.content, b"body");
    assert_eq!(entry.total_size, (b"title".len() + b"body".len()) as i64);
    assert_eq!(entry.entry_date, entry.created_at.date_naive());

    let page = journal::entries::list(&pool, user.id, notebook.id, None, ListOrder::Desc, 50)
        .await
        .expect("list");
    assert_eq!(page.entries.len(), 1);
    assert!(page.next_cursor.is_none());

    let fetched = journal::entries::get(&pool, user.id, entry.id)
        .await
        .expect("get");
    assert_eq!(fetched.content, b"body");

    journal::entries::update(&pool, user.id, entry.id, Some(b"new-title"), None, None)
        .await
        .expect("update");
    let updated = journal::entries::get(&pool, user.id, entry.id)
        .await
        .expect("get updated");
    assert_eq!(updated.title, b"new-title");
    assert_eq!(
        updated.total_size,
        (b"new-title".len() + b"body".len()) as i64
    );
    assert!(updated.updated_at.is_some());

    let dated = chrono::NaiveDate::from_ymd_opt(2020, 1, 15).expect("date");
    journal::entries::update(&pool, user.id, entry.id, None, None, Some(dated))
        .await
        .expect("update date");
    let dated_entry = journal::entries::get(&pool, user.id, entry.id)
        .await
        .expect("get dated");
    assert_eq!(dated_entry.entry_date, dated);
    assert_eq!(dated_entry.title, b"new-title");

    for i in 1..20 {
        journal::journals::create(
            &pool,
            user.id,
            &format!("ws-{i}"),
            b"salt",
            b"dek-bytes",
            None,
        )
        .await
        .expect("journal n");
    }
    let err =
        journal::journals::create(&pool, user.id, "too-many", b"salt", b"dek-bytes", None)
            .await
            .expect_err("limit");
    match err {
        AppError::BadRequest(message) => {
            assert!(message.contains("more than 20 journals"));
        }
        other => panic!("expected bad request, got {other:?}"),
    }
}
