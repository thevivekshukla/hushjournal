use chrono::{DateTime, Utc};
use errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{ENTRY_CONTENT_MAX, ENTRY_TITLE_MAX, map_db, require_bytes_max};

pub const ENTRY_LIST_DEFAULT_LIMIT: i64 = 50;
pub const ENTRY_LIST_MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    #[default]
    Desc,
    Asc,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryPage {
    pub entries: Vec<EntrySummary>,
    pub next_cursor: Option<Uuid>,
}

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

pub fn list_limit(limit: Option<i64>) -> Result<i64, AppError> {
    match limit {
        None => Ok(ENTRY_LIST_DEFAULT_LIMIT),
        Some(n) if (1..=ENTRY_LIST_MAX_LIMIT).contains(&n) => Ok(n),
        Some(_) => Err(AppError::BadRequest(
            "limit must be between 1 and 100".into(),
        )),
    }
}

pub async fn list(
    pool: &SqlitePool,
    user_id: Uuid,
    shelf_id: Uuid,
    cursor: Option<Uuid>,
    order: ListOrder,
    limit: i64,
) -> Result<EntryPage, AppError> {
    crate::shelves::get(pool, user_id, shelf_id).await?;
    let fetch = limit + 1;
    let mut entries = match (order, cursor) {
        (ListOrder::Desc, None) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, total_size,
                    created_at as "created_at!: DateTime<Utc>",
                    updated_at as "updated_at: DateTime<Utc>"
                FROM entries
                WHERE shelf_id = $1
                ORDER BY id DESC
                LIMIT $2
            "#,
            shelf_id,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Desc, Some(cursor)) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, total_size,
                    created_at as "created_at!: DateTime<Utc>",
                    updated_at as "updated_at: DateTime<Utc>"
                FROM entries
                WHERE shelf_id = $1 AND id < $2
                ORDER BY id DESC
                LIMIT $3
            "#,
            shelf_id,
            cursor,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Asc, None) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, total_size,
                    created_at as "created_at!: DateTime<Utc>",
                    updated_at as "updated_at: DateTime<Utc>"
                FROM entries
                WHERE shelf_id = $1
                ORDER BY id ASC
                LIMIT $2
            "#,
            shelf_id,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Asc, Some(cursor)) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, total_size,
                    created_at as "created_at!: DateTime<Utc>",
                    updated_at as "updated_at: DateTime<Utc>"
                FROM entries
                WHERE shelf_id = $1 AND id > $2
                ORDER BY id ASC
                LIMIT $3
            "#,
            shelf_id,
            cursor,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
    };
    let next_cursor = if (entries.len() as i64) > limit {
        entries.truncate(limit as usize);
        entries.last().map(|entry| entry.id)
    } else {
        None
    };
    Ok(EntryPage {
        entries,
        next_cursor,
    })
}

pub async fn get(pool: &SqlitePool, user_id: Uuid, id: Uuid) -> Result<Entry, AppError> {
    sqlx::query_as!(
        Entry,
        r#"
            SELECT e.id as "id!: Uuid", e.shelf_id as "shelf_id!: Uuid", e.title, e.content,
                e.total_size,
                e.created_at as "created_at!: DateTime<Utc>",
                e.updated_at as "updated_at: DateTime<Utc>"
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
    pool: &SqlitePool,
    user_id: Uuid,
    shelf_id: Uuid,
    title: &[u8],
    content: &[u8],
) -> Result<Entry, AppError> {
    validate_title(title)?;
    validate_content(content)?;
    let id = Uuid::now_v7();

    sqlx::query_as!(
        Entry,
        r#"
            INSERT INTO entries (id, shelf_id, title, content)
            SELECT $1, $2, $3, $4
            FROM shelves s
            JOIN workspaces w ON w.id = s.workspace_id
            WHERE s.id = $2 AND w.user_id = $5
            RETURNING id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, content, total_size,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        id,
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
    pool: &SqlitePool,
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
            UPDATE entries
            SET title = COALESCE($3, title),
                content = COALESCE($4, content)
            WHERE id = $1
              AND shelf_id IN (
                    SELECT s.id
                    FROM shelves s
                    JOIN workspaces w ON w.id = s.workspace_id
                    WHERE w.user_id = $2
              )
            RETURNING id as "id!: Uuid", shelf_id as "shelf_id!: Uuid", title, content, total_size,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
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

pub async fn delete(pool: &SqlitePool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"
            DELETE FROM entries
            WHERE id = $1
              AND shelf_id IN (
                    SELECT s.id
                    FROM shelves s
                    JOIN workspaces w ON w.id = s.workspace_id
                    WHERE w.user_id = $2
              )
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
