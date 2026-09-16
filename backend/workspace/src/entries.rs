use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{ENTRY_CONTENT_MAX, ENTRY_TITLE_MAX, map_db, require_bytes_max};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Entry {
    pub id: Uuid,
    pub shelf_id: Uuid,
    #[serde(with = "crate::b64")]
    pub title: Vec<u8>,
    #[serde(with = "crate::b64")]
    pub content: Vec<u8>,
    pub total_size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EntrySummary {
    pub id: Uuid,
    pub shelf_id: Uuid,
    #[serde(with = "crate::b64")]
    pub title: Vec<u8>,
    pub total_size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub fn validate_title(title: &[u8]) -> Result<(), AppError> {
    require_bytes_max(title, "title", ENTRY_TITLE_MAX)
}

pub fn validate_content(content: &[u8]) -> Result<(), AppError> {
    if content.len() > ENTRY_CONTENT_MAX {
        return Err(AppError::BadRequest("content is too large".into()));
    }
    Ok(())
}

pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    shelf_id: Uuid,
) -> Result<Vec<EntrySummary>, AppError> {
    crate::shelves::get(pool, user_id, shelf_id).await?;
    sqlx::query_as!(
        EntrySummary,
        r#"
            SELECT id, shelf_id, title, total_size, created_at, updated_at
            FROM entries
            WHERE shelf_id = $1
            ORDER BY created_at DESC, id DESC
        "#,
        shelf_id,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db)
}

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Entry, AppError> {
    sqlx::query_as!(
        Entry,
        r#"
            SELECT e.id, e.shelf_id, e.title, e.content, e.total_size,
                e.created_at, e.updated_at
            FROM entries e
            JOIN shelves s ON s.id = e.shelf_id
            JOIN workspaces w ON w.id = s.workspace_id
            WHERE e.id = $1 AND w.user_id = $2
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
    shelf_id: Uuid,
    title: &[u8],
    content: &[u8],
) -> Result<Entry, AppError> {
    validate_title(title)?;
    validate_content(content)?;

    sqlx::query_as!(
        Entry,
        r#"
            INSERT INTO entries (shelf_id, title, content)
            SELECT $1, $2, $3
            FROM shelves s
            JOIN workspaces w ON w.id = s.workspace_id
            WHERE s.id = $1 AND w.user_id = $4
            RETURNING id, shelf_id, title, content, total_size, created_at, updated_at
        "#,
        shelf_id,
        title,
        content,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?
    .ok_or(AppError::NotFound)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    title: Option<&[u8]>,
    content: Option<&[u8]>,
) -> Result<Entry, AppError> {
    if title.is_none() && content.is_none() {
        return Err(AppError::BadRequest("no fields to update".into()));
    }
    if let Some(title) = title {
        validate_title(title)?;
    }
    if let Some(content) = content {
        validate_content(content)?;
    }

    sqlx::query_as!(
        Entry,
        r#"
            UPDATE entries e
            SET title = COALESCE($3, e.title),
                content = COALESCE($4, e.content)
            FROM shelves s, workspaces w
            WHERE e.id = $1
                AND e.shelf_id = s.id
                AND s.workspace_id = w.id
                AND w.user_id = $2
            RETURNING e.id, e.shelf_id, e.title, e.content, e.total_size,
                e.created_at, e.updated_at
        "#,
        id,
        user_id,
        title,
        content,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?
    .ok_or(AppError::NotFound)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"
            DELETE FROM entries e
            USING shelves s, workspaces w
            WHERE e.id = $1
                AND e.shelf_id = s.id
                AND s.workspace_id = w.id
                AND w.user_id = $2
        "#,
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
