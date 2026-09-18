use chrono::{DateTime, NaiveDate, Utc};
use errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub notebook_id: Uuid,
    #[serde(with = "crate::b64")]
    pub title: Vec<u8>,
    #[serde(with = "crate::b64")]
    pub content: Vec<u8>,
    pub total_size: i64,
    pub entry_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EntrySummary {
    pub id: Uuid,
    pub notebook_id: Uuid,
    #[serde(with = "crate::b64")]
    pub title: Vec<u8>,
    pub total_size: i64,
    pub entry_date: NaiveDate,
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
    pool: &PgPool,
    user_id: Uuid,
    notebook_id: Uuid,
    cursor: Option<Uuid>,
    order: ListOrder,
    limit: i64,
) -> Result<EntryPage, AppError> {
    crate::notebooks::get(pool, user_id, notebook_id).await?;
    let fetch = limit + 1;
    let mut entries = match (order, cursor) {
        (ListOrder::Desc, None) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id, notebook_id, title, total_size, entry_date, created_at, updated_at
                FROM entries
                WHERE notebook_id = $1
                ORDER BY id DESC
                LIMIT $2
            "#,
            notebook_id,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Desc, Some(cursor)) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id, notebook_id, title, total_size, entry_date, created_at, updated_at
                FROM entries
                WHERE notebook_id = $1 AND id < $2
                ORDER BY id DESC
                LIMIT $3
            "#,
            notebook_id,
            cursor,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Asc, None) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id, notebook_id, title, total_size, entry_date, created_at, updated_at
                FROM entries
                WHERE notebook_id = $1
                ORDER BY id ASC
                LIMIT $2
            "#,
            notebook_id,
            fetch,
        )
        .fetch_all(pool)
        .await
        .map_err(map_db)?,
        (ListOrder::Asc, Some(cursor)) => sqlx::query_as!(
            EntrySummary,
            r#"
                SELECT id, notebook_id, title, total_size, entry_date, created_at, updated_at
                FROM entries
                WHERE notebook_id = $1 AND id > $2
                ORDER BY id ASC
                LIMIT $3
            "#,
            notebook_id,
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

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Entry, AppError> {
    sqlx::query_as!(
        Entry,
        r#"
            SELECT e.id, e.notebook_id, e.title, e.content, e.total_size, e.entry_date,
                e.created_at, e.updated_at
            FROM entries e
            JOIN notebooks s ON s.id = e.notebook_id
            JOIN journals w ON w.id = s.journal_id
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
    notebook_id: Uuid,
    title: &[u8],
    content: &[u8],
    entry_date: Option<NaiveDate>,
) -> Result<Entry, AppError> {
    validate_title(title)?;
    validate_content(content)?;

    sqlx::query_as!(
        Entry,
        r#"
            INSERT INTO entries (notebook_id, title, content, entry_date)
            SELECT $1, $2, $3, COALESCE($4, CURRENT_DATE)
            FROM notebooks s
            JOIN journals w ON w.id = s.journal_id
            WHERE s.id = $1 AND w.user_id = $5
            RETURNING id, notebook_id, title, content, total_size, entry_date, created_at, updated_at
        "#,
        notebook_id,
        title,
        content,
        entry_date,
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
    entry_date: Option<NaiveDate>,
) -> Result<Entry, AppError> {
    if title.is_none() && content.is_none() && entry_date.is_none() {
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
                content = COALESCE($4, e.content),
                entry_date = COALESCE($5, e.entry_date)
            FROM notebooks s, journals w
            WHERE e.id = $1
                AND e.notebook_id = s.id
                AND s.journal_id = w.id
                AND w.user_id = $2
            RETURNING e.id, e.notebook_id, e.title, e.content, e.total_size, e.entry_date,
                e.created_at, e.updated_at
        "#,
        id,
        user_id,
        title,
        content,
        entry_date,
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
            USING notebooks s, journals w
            WHERE e.id = $1
                AND e.notebook_id = s.id
                AND s.journal_id = w.id
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
