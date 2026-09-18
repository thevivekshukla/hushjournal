use sqlx::PgPool;

#[sqlx::test(migrations = "../db/migrations")]
async fn password_signup_then_login(pool: PgPool) {
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

#[sqlx::test(migrations = "../db/migrations")]
async fn password_signup_rejects_taken_username(pool: PgPool) {
    user::signup_with_password(&pool, "grace", "password1")
        .await
        .expect("signup");
    let err = user::signup_with_password(&pool, "Grace", "password2")
        .await
        .expect_err("taken");
    assert!(matches!(err, errors::AppError::Conflict));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn password_login_rejects_wrong_password(pool: PgPool) {
    user::signup_with_password(&pool, "hopper", "password1")
        .await
        .expect("signup");
    let err = user::login_with_password(&pool, "hopper", "password2")
        .await
        .expect_err("wrong");
    assert!(matches!(err, errors::AppError::BadRequest(_)));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn password_login_rejects_inactive_user(pool: PgPool) {
    let created = user::signup_with_password(&pool, "katherine", "password1")
        .await
        .expect("signup");
    sqlx::query!(
        "UPDATE users SET is_active = false WHERE id = $1",
        created.id
    )
    .execute(&pool)
    .await
    .expect("deactivate");

    let err = user::login_with_password(&pool, "katherine", "password1")
        .await
        .expect_err("inactive");
    assert!(matches!(err, errors::AppError::Forbidden));
}
