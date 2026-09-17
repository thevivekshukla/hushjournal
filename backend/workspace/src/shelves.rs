use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{MAX_ICON_LEN, SHELF_NAME_MAX, map_db, require_bytes_max};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Shelf {
    pub id: Uuid,
    pub workspace_id: Uuid,
    #[serde(with = "crate::b64")]
    pub name: Vec<u8>,
    pub icon: Option<String>,
    pub total_shelf_size: i64,
    pub size_last_calculated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub fn normalize_icon(icon: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(icon) = icon.map(str::trim).filter(|icon| !icon.is_empty()) else {
        return Ok(None);
    };
    if icon.chars().count() > MAX_ICON_LEN {
        return Err(AppError::BadRequest("icon is too long".into()));
    }
    Ok(Some(icon.to_string()))
}

pub async fn list(
    pool: &SqlitePool,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<Vec<Shelf>, AppError> {
    crate::workspaces::get(pool, user_id, workspace_id).await?;
    sqlx::query_as!(
        Shelf,
        r#"
            SELECT id as "id!: Uuid", workspace_id as "workspace_id!: Uuid", name, icon,
                total_shelf_size,
                size_last_calculated_at as "size_last_calculated_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM shelves
            WHERE workspace_id = $1
            ORDER BY created_at ASC, id ASC
        "#,
        workspace_id,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db)
}

pub async fn get(pool: &SqlitePool, user_id: Uuid, id: Uuid) -> Result<Shelf, AppError> {
    sqlx::query_as!(
        Shelf,
        r#"
            SELECT s.id as "id!: Uuid", s.workspace_id as "workspace_id!: Uuid", s.name, s.icon,
                s.total_shelf_size,
                s.size_last_calculated_at as "size_last_calculated_at: DateTime<Utc>",
                s.created_at as "created_at!: DateTime<Utc>",
                s.updated_at as "updated_at: DateTime<Utc>"
            FROM shelves s
            JOIN workspaces w ON w.id = s.workspace_id
            WHERE s.id = $1 AND w.user_id = $2
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
    workspace_id: Uuid,
    name: &[u8],
    icon: Option<&str>,
) -> Result<Shelf, AppError> {
    require_bytes_max(name, "name", SHELF_NAME_MAX)?;
    let icon = normalize_icon(icon)?;
    let id = Uuid::now_v7();

    sqlx::query_as!(
        Shelf,
        r#"
            INSERT INTO shelves (id, workspace_id, name, icon)
            SELECT $1, $2, $3, $4
            FROM workspaces
            WHERE id = $2 AND user_id = $5
            RETURNING id as "id!: Uuid", workspace_id as "workspace_id!: Uuid", name, icon,
                total_shelf_size,
                size_last_calculated_at as "size_last_calculated_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        id,
        workspace_id,
        name,
        icon,
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
    name: Option<&[u8]>,
    icon_set: bool,
    icon: Option<&str>,
) -> Result<Shelf, AppError> {
    if name.is_none() && !icon_set {
        return Err(AppError::BadRequest("no fields to update".into()));
    }
    if let Some(name) = name {
        require_bytes_max(name, "name", SHELF_NAME_MAX)?;
    }
    let icon = if icon_set {
        normalize_icon(icon)?
    } else {
        None
    };

    sqlx::query_as!(
        Shelf,
        r#"
            UPDATE shelves
            SET name = COALESCE($3, name),
                icon = CASE WHEN $4 THEN $5 ELSE icon END
            WHERE id = $1
              AND workspace_id IN (SELECT id FROM workspaces WHERE user_id = $2)
            RETURNING id as "id!: Uuid", workspace_id as "workspace_id!: Uuid", name, icon,
                total_shelf_size,
                size_last_calculated_at as "size_last_calculated_at: DateTime<Utc>",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
        "#,
        id,
        user_id,
        name,
        icon_set,
        icon,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db)?
    .ok_or(AppError::NotFound)
}

pub async fn delete(pool: &SqlitePool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"
            DELETE FROM shelves
            WHERE id = $1
              AND workspace_id IN (SELECT id FROM workspaces WHERE user_id = $2)
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
