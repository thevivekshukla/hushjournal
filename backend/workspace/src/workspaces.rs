use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ENCRYPTED_DEK_MAX, KEY_SALT_MAX, MAX_NAME_LEN, PASSPHRASE_HINT_MAX, map_db, require_bytes_max,
};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(with = "crate::b64")]
    pub key_salt: Vec<u8>,
    #[serde(with = "crate::b64")]
    pub encrypted_dek: Vec<u8>,
    pub passphrase_hint: Option<String>,
    pub mask: bool,
    pub total_workspace_size: i64,
    pub size_last_calculated_at: Option<DateTime<Utc>>,
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

pub fn normalize_hint(hint: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(hint) = hint else {
        return Ok(None);
    };
    let hint = hint.trim();
    if hint.is_empty() {
        return Ok(None);
    }
    if hint.chars().count() > PASSPHRASE_HINT_MAX {
        return Err(AppError::BadRequest("passphrase hint is too long".into()));
    }
    Ok(Some(hint.to_string()))
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Workspace>, AppError> {
    sqlx::query_as!(
        Workspace,
        r#"
            SELECT id, user_id, name, key_salt, encrypted_dek, passphrase_hint, mask,
                total_workspace_size, size_last_calculated_at, created_at, updated_at
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
            SELECT id, user_id, name, key_salt, encrypted_dek, passphrase_hint, mask,
                total_workspace_size, size_last_calculated_at, created_at, updated_at
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
    passphrase_hint: Option<&str>,
) -> Result<Workspace, AppError> {
    let name = normalize_name(name)?;
    require_bytes_max(key_salt, "key_salt", KEY_SALT_MAX)?;
    require_bytes_max(encrypted_dek, "encrypted_dek", ENCRYPTED_DEK_MAX)?;
    let passphrase_hint = normalize_hint(passphrase_hint)?;

    sqlx::query_as!(
        Workspace,
        r#"
            INSERT INTO workspaces (user_id, name, key_salt, encrypted_dek, passphrase_hint)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, key_salt, encrypted_dek, passphrase_hint, mask,
                total_workspace_size, size_last_calculated_at, created_at, updated_at
        "#,
        user_id,
        name,
        key_salt,
        encrypted_dek,
        passphrase_hint,
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
    passphrase_hint: Option<&str>,
    mask: Option<bool>,
) -> Result<Workspace, AppError> {
    if name.is_none()
        && key_salt.is_none()
        && encrypted_dek.is_none()
        && passphrase_hint.is_none()
        && mask.is_none()
    {
        return Err(AppError::BadRequest("no fields to update".into()));
    }
    if key_salt.is_some() != encrypted_dek.is_some() {
        return Err(AppError::BadRequest(
            "key_salt and encrypted_dek must be updated together".into(),
        ));
    }
    let name = name.map(normalize_name).transpose()?;
    if let Some(key_salt) = key_salt {
        require_bytes_max(key_salt, "key_salt", KEY_SALT_MAX)?;
    }
    if let Some(encrypted_dek) = encrypted_dek {
        require_bytes_max(encrypted_dek, "encrypted_dek", ENCRYPTED_DEK_MAX)?;
    }
    let set_hint = passphrase_hint.is_some();
    let passphrase_hint = normalize_hint(passphrase_hint)?;

    sqlx::query_as!(
        Workspace,
        r#"
            UPDATE workspaces
            SET name = COALESCE($3, name),
                key_salt = COALESCE($4, key_salt),
                encrypted_dek = COALESCE($5, encrypted_dek),
                passphrase_hint = CASE WHEN $6 THEN $7 ELSE passphrase_hint END,
                mask = COALESCE($8, mask)
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, key_salt, encrypted_dek, passphrase_hint, mask,
                total_workspace_size, size_last_calculated_at, created_at, updated_at
        "#,
        id,
        user_id,
        name,
        key_salt,
        encrypted_dek,
        set_hint,
        passphrase_hint,
        mask,
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
