use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{DateTime, Utc};
use errors::AppError;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::{MAX_PASSWORD_LEN, MAX_USERNAME_LEN, MIN_PASSWORD_LEN, MIN_USERNAME_LEN, User, map_db};

const INVALID_CREDENTIALS: &str = "invalid username or password";

pub async fn signup_with_password(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<User, AppError> {
    let username = normalize_username(username)?;
    let password = normalize_password(password)?;
    let password_hash = hash_password(password)?;
    let id = Uuid::now_v7();
    let name = username.clone();

    sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (id, name, username, password_hash, last_login_at)
            VALUES ($1, $2, $3, $4, unixepoch())
            RETURNING id as "id!: Uuid", name, username, email,
                is_email_verified as "is_email_verified!: bool",
                email_verified_at as "email_verified_at: DateTime<Utc>",
                google_email, google_account_id, google_avatar_url,
                is_active as "is_active!: bool",
                last_login_at as "last_login_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        id,
        name,
        username,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .map_err(map_db)
}

pub async fn login_with_password(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<User, AppError> {
    let username = normalize_username(username)?;
    let password = normalize_password(password)?;
    let row = sqlx::query!(
        r#"
            SELECT id as "id!: Uuid", password_hash, is_active as "is_active!: bool"
            FROM users
            WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::BadRequest(INVALID_CREDENTIALS.into()));
    };
    let Some(password_hash) = row.password_hash.filter(|hash| !hash.is_empty()) else {
        return Err(AppError::BadRequest(INVALID_CREDENTIALS.into()));
    };
    verify_password(password, &password_hash)?;
    if !row.is_active {
        return Err(AppError::Forbidden);
    }

    sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET last_login_at = unixepoch()
            WHERE id = $1
            RETURNING id as "id!: Uuid", name, username, email,
                is_email_verified as "is_email_verified!: bool",
                email_verified_at as "email_verified_at: DateTime<Utc>",
                google_email, google_account_id, google_avatar_url,
                is_active as "is_active!: bool",
                last_login_at as "last_login_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        row.id,
    )
    .fetch_one(pool)
    .await
    .map_err(map_db)
}

fn normalize_username(username: &str) -> Result<String, AppError> {
    let username = username.trim().to_lowercase();
    let chars: Vec<char> = username.chars().collect();
    if chars.len() < MIN_USERNAME_LEN || chars.len() > MAX_USERNAME_LEN {
        return Err(AppError::BadRequest(
            "username must be 3–32 letters, numbers, or underscores".into(),
        ));
    }
    let Some(first) = chars.first() else {
        return Err(AppError::BadRequest("username is required".into()));
    };
    if !first.is_ascii_lowercase()
        || !chars
            .iter()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || *ch == '_')
    {
        return Err(AppError::BadRequest(
            "username must start with a letter and use only letters, numbers, or underscores"
                .into(),
        ));
    }
    Ok(username)
}

fn normalize_password(password: &str) -> Result<&str, AppError> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    if password.chars().count() > MAX_PASSWORD_LEN {
        return Err(AppError::BadRequest("password is too long".into()));
    }
    Ok(password)
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Other(anyhow::anyhow!("password hash failed: {err}")))
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), AppError> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|_| AppError::BadRequest(INVALID_CREDENTIALS.into()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::BadRequest(INVALID_CREDENTIALS.into()))
}
