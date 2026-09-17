use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

pub const MAX_NAME_LEN: usize = 255;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub is_email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub google_email: Option<String>,
    #[serde(skip_serializing)]
    pub google_account_id: Option<String>,
    pub google_avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct GoogleAccount {
    pub google_account_id: String,
    pub name: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
}

pub fn normalize_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::BadRequest("name is too long".into()));
    }
    Ok(name.to_string())
}

pub fn display_name(name: Option<&str>, email: Option<&str>) -> String {
    if let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) {
        return name.chars().take(MAX_NAME_LEN).collect();
    }
    if let Some(email) = email.map(str::trim).filter(|email| !email.is_empty()) {
        let local = email.split('@').next().unwrap_or(email);
        if !local.is_empty() {
            return local.chars().take(MAX_NAME_LEN).collect();
        }
    }
    "User".into()
}

pub async fn get_by_id(pool: &SqlitePool, id: Uuid) -> Result<User, AppError> {
    let user = get_by_id_unchecked(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    require_active(&user)?;
    Ok(user)
}

pub async fn login_with_google(
    pool: &SqlitePool,
    account: &GoogleAccount,
) -> Result<User, AppError> {
    if let Some(existing) = get_by_google_account_id(pool, &account.google_account_id).await? {
        require_active(&existing)?;
        return update_google_login(pool, existing.id, account).await;
    }

    match insert_google_user(pool, account).await {
        Ok(user) => Ok(user),
        Err(AppError::Conflict) => {
            let existing = get_by_google_account_id(pool, &account.google_account_id)
                .await?
                .ok_or(AppError::Conflict)?;
            require_active(&existing)?;
            update_google_login(pool, existing.id, account).await
        }
        Err(err) => Err(err),
    }
}

pub async fn update_name(pool: &SqlitePool, id: Uuid, name: &str) -> Result<User, AppError> {
    let name = normalize_name(name)?;
    let user = sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET name = $2
            WHERE id = $1
            RETURNING id as "id!: Uuid", name, email,
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
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?;
    let user = user.ok_or(AppError::NotFound)?;
    require_active(&user)?;
    Ok(user)
}

pub async fn delete_by_id(pool: &SqlitePool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

async fn get_by_id_unchecked(pool: &SqlitePool, id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as!(
        User,
        r#"
            SELECT id as "id!: Uuid", name, email,
                is_email_verified as "is_email_verified!: bool",
                email_verified_at as "email_verified_at: DateTime<Utc>",
                google_email, google_account_id, google_avatar_url,
                is_active as "is_active!: bool",
                last_login_at as "last_login_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM users
            WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

async fn get_by_google_account_id(
    pool: &SqlitePool,
    google_account_id: &str,
) -> Result<Option<User>, AppError> {
    sqlx::query_as!(
        User,
        r#"
            SELECT id as "id!: Uuid", name, email,
                is_email_verified as "is_email_verified!: bool",
                email_verified_at as "email_verified_at: DateTime<Utc>",
                google_email, google_account_id, google_avatar_url,
                is_active as "is_active!: bool",
                last_login_at as "last_login_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM users
            WHERE google_account_id = $1
        "#,
        google_account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)
}

async fn insert_google_user(pool: &SqlitePool, account: &GoogleAccount) -> Result<User, AppError> {
    let id = Uuid::now_v7();
    let name = display_name(Some(&account.name), account.email.as_deref());
    sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (
                id, name, email, is_email_verified, email_verified_at,
                google_email, google_account_id, google_avatar_url, last_login_at
            )
            VALUES ($1, $2, $3, $4, CASE WHEN $4 THEN unixepoch() ELSE NULL END, $5, $6, $7, unixepoch())
            RETURNING id as "id!: Uuid", name, email,
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
        account.email.as_deref(),
        account.email_verified,
        account.email.as_deref(),
        &account.google_account_id,
        account.avatar_url.as_deref(),
    )
    .fetch_one(pool)
    .await
    .map_err(map_db)
}

async fn update_google_login(
    pool: &SqlitePool,
    id: Uuid,
    account: &GoogleAccount,
) -> Result<User, AppError> {
    sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET google_email = $2, google_avatar_url = $3, last_login_at = unixepoch()
            WHERE id = $1
            RETURNING id as "id!: Uuid", name, email,
                is_email_verified as "is_email_verified!: bool",
                email_verified_at as "email_verified_at: DateTime<Utc>",
                google_email, google_account_id, google_avatar_url,
                is_active as "is_active!: bool",
                last_login_at as "last_login_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        id,
        account.email,
        account.avatar_url,
    )
    .fetch_one(pool)
    .await
    .map_err(map_db)
}

fn require_active(user: &User) -> Result<(), AppError> {
    if user.is_active {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn map_db(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.is_unique_violation()
    {
        return AppError::Conflict;
    }
    AppError::from(err)
}
