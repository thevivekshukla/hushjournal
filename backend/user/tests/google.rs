use user::GoogleAccount;

fn account(id: &str, name: &str, email: &str) -> GoogleAccount {
    GoogleAccount {
        google_account_id: id.into(),
        name: name.into(),
        email: Some(email.into()),
        email_verified: true,
        avatar_url: Some("https://example.com/a.png".into()),
    }
}

#[tokio::test]
async fn google_login_creates_and_updates_without_renaming() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    let created =
        user::login_with_google(&pool, &account("g-1", "Ada Lovelace", "ada@example.com"))
            .await
            .expect("create");
    assert_eq!(created.name, "Ada Lovelace");
    assert_eq!(created.google_email.as_deref(), Some("ada@example.com"));
    assert!(created.is_active);

    let updated = user::login_with_google(
        &pool,
        &account("g-1", "Ignored Name", "ada.new@example.com"),
    )
    .await
    .expect("update");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Ada Lovelace");
    assert_eq!(updated.google_email.as_deref(), Some("ada.new@example.com"));
}

#[tokio::test]
async fn google_login_rejects_inactive_user() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    let created = user::login_with_google(&pool, &account("g-2", "Grace", "grace@example.com"))
        .await
        .expect("create");
    sqlx::query("UPDATE users SET is_active = FALSE WHERE id = ?")
        .bind(created.id)
        .execute(&pool)
        .await
        .expect("deactivate");

    let err = user::login_with_google(&pool, &account("g-2", "Grace", "grace@example.com"))
        .await
        .expect_err("inactive");
    assert!(matches!(err, errors::AppError::Forbidden));
}
