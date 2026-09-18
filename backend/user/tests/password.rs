#[tokio::test]
async fn password_signup_then_login() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    let created = user::signup_with_password(&pool, "Ada_Lovelace", "correct horse")
        .await
        .expect("signup");
    assert_eq!(created.username.as_deref(), Some("ada_lovelace"));
    assert_eq!(created.name, "ada_lovelace");
    assert!(created.is_active);

    let logged_in = user::login_with_password(&pool, "ada_lovelace", "correct horse")
        .await
        .expect("login");
    assert_eq!(logged_in.id, created.id);
}

#[tokio::test]
async fn password_signup_rejects_taken_username() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    user::signup_with_password(&pool, "grace", "password1")
        .await
        .expect("signup");
    let err = user::signup_with_password(&pool, "Grace", "password2")
        .await
        .expect_err("taken");
    assert!(matches!(err, errors::AppError::Conflict));
}

#[tokio::test]
async fn password_login_rejects_wrong_password() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    user::signup_with_password(&pool, "hopper", "password1")
        .await
        .expect("signup");
    let err = user::login_with_password(&pool, "hopper", "password2")
        .await
        .expect_err("wrong");
    assert!(matches!(err, errors::AppError::BadRequest(_)));
}

#[tokio::test]
async fn password_login_rejects_inactive_user() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    let created = user::signup_with_password(&pool, "katherine", "password1")
        .await
        .expect("signup");
    sqlx::query("UPDATE users SET is_active = FALSE WHERE id = ?")
        .bind(created.id)
        .execute(&pool)
        .await
        .expect("deactivate");

    let err = user::login_with_password(&pool, "katherine", "password1")
        .await
        .expect_err("inactive");
    assert!(matches!(err, errors::AppError::Forbidden));
}
