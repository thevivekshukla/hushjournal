use chrono::{DateTime, Utc};
use errors::AppError;
use serde::Serialize;
use sqlx::PgPool;
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
    pool: &PgPool,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<Vec<Shelf>, AppError> {
    crate::workspaces::get(pool, user_id, workspace_id).await?;
    sqlx::query_as!(
        Shelf,
        r#"
            SELECT id, workspace_id, name, icon, total_shelf_size,
                size_last_calculated_at, created_at, updated_at
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

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Shelf, AppError> {
    sqlx::query_as!(
        Shelf,
        r#"
            SELECT s.id, s.workspace_id, s.name, s.icon, s.total_shelf_size,
                s.size_last_calculated_at, s.created_at, s.updated_at
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
    pool: &PgPool,
    user_id: Uuid,
    workspace_id: Uuid,
    name: &[u8],
    icon: Option<&str>,
) -> Result<Shelf, AppError> {
    require_bytes_max(name, "name", SHELF_NAME_MAX)?;
    let icon = normalize_icon(icon)?;

    sqlx::query_as!(
        Shelf,
        r#"
            INSERT INTO shelves (workspace_id, name, icon)
            SELECT $1, $2, $3
            FROM workspaces
            WHERE id = $1 AND user_id = $4
            RETURNING id, workspace_id, name, icon, total_shelf_size,
                size_last_calculated_at, created_at, updated_at
        "#,
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
    pool: &PgPool,
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
            UPDATE shelves s
            SET name = COALESCE($3, s.name),
                icon = CASE WHEN $4 THEN $5 ELSE s.icon END
            FROM workspaces w
            WHERE s.id = $1 AND s.workspace_id = w.id AND w.user_id = $2
            RETURNING s.id, s.workspace_id, s.name, s.icon, s.total_shelf_size,
                s.size_last_calculated_at, s.created_at, s.updated_at
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

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        r#"
            DELETE FROM shelves s
            USING workspaces w
            WHERE s.id = $1 AND s.workspace_id = w.id AND w.user_id = $2
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
