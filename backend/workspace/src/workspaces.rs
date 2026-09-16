use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{ENCRYPTED_DEK_MAX, KEY_SALT_MAX, MAX_NAME_LEN, map_db, require_bytes_max};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(with = "crate::b64")]
    pub key_salt: Vec<u8>,
    #[serde(with = "crate::b64")]
    pub encrypted_dek: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
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

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Workspace>, AppError> {
    sqlx::query_as!(
        Workspace,
        r#"
            SELECT id, user_id, name, key_salt, encrypted_dek, created_at, updated_at
            FROM workspaces
            WHERE user_id = $1
            ORDER BY created_at DESC, id DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db)
}

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Workspace, AppError> {
    sqlx::query_as!(
        Workspace,
        r#"
            SELECT id, user_id, name, key_salt, encrypted_dek, created_at, updated_at
            FROM workspaces
            WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?
    .ok_or(AppError::NotFound)
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    key_salt: &[u8],
    encrypted_dek: &[u8],
) -> Result<Workspace, AppError> {
    let name = normalize_name(name)?;
    require_bytes_max(key_salt, "key_salt", KEY_SALT_MAX)?;
    require_bytes_max(encrypted_dek, "encrypted_dek", ENCRYPTED_DEK_MAX)?;

    sqlx::query_as!(
        Workspace,
        r#"
            INSERT INTO workspaces (user_id, name, key_salt, encrypted_dek)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, key_salt, encrypted_dek, created_at, updated_at
        "#,
        user_id,
        name,
        key_salt,
        encrypted_dek,
    )
    .fetch_one(pool)
    .await
    .map_err(map_db)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    name: Option<&str>,
    key_salt: Option<&[u8]>,
    encrypted_dek: Option<&[u8]>,
) -> Result<Workspace, AppError> {
    if name.is_none() && key_salt.is_none() && encrypted_dek.is_none() {
        return Err(AppError::BadRequest("no fields to update".into()));
    }
    let name = name.map(normalize_name).transpose()?;
    if let Some(key_salt) = key_salt {
        require_bytes_max(key_salt, "key_salt", KEY_SALT_MAX)?;
    }
    if let Some(encrypted_dek) = encrypted_dek {
        require_bytes_max(encrypted_dek, "encrypted_dek", ENCRYPTED_DEK_MAX)?;
    }

    sqlx::query_as!(
        Workspace,
        r#"
            UPDATE workspaces
            SET name = COALESCE($3, name),
                key_salt = COALESCE($4, key_salt),
                encrypted_dek = COALESCE($5, encrypted_dek)
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, key_salt, encrypted_dek, created_at, updated_at
        "#,
        id,
        user_id,
        name,
        key_salt,
        encrypted_dek,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?
    .ok_or(AppError::NotFound)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        "DELETE FROM workspaces WHERE id = $1 AND user_id = $2",
        id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(map_db)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
