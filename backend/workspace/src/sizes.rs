use errors::AppError;
use sqlx::SqlitePool;

use crate::map_db;

pub async fn recalculate_stale_shelf_sizes(pool: &SqlitePool) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
            UPDATE shelves
            SET total_shelf_size = (
                    SELECT COALESCE(SUM(e.total_size), 0)
                    FROM entries e
                    WHERE e.shelf_id = shelves.id
                ),
                size_last_calculated_at = unixepoch()
            WHERE size_last_calculated_at IS NULL
               OR EXISTS (
                    SELECT 1
                    FROM entries e
                    WHERE e.shelf_id = shelves.id
                      AND (
                          e.created_at > shelves.size_last_calculated_at
                          OR e.updated_at > shelves.size_last_calculated_at
                      )
                )
        "#
    )
    .execute(pool)
    .await
    .map_err(map_db)?;
    Ok(result.rows_affected())
}

pub async fn recalculate_stale_workspace_sizes(pool: &SqlitePool) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
            UPDATE workspaces
            SET total_workspace_size = (
                    SELECT COALESCE(SUM(s.total_shelf_size), 0)
                    FROM shelves s
                    WHERE s.workspace_id = workspaces.id
                ),
                size_last_calculated_at = unixepoch()
            WHERE size_last_calculated_at IS NULL
               OR EXISTS (
                    SELECT 1
                    FROM shelves s
                    WHERE s.workspace_id = workspaces.id
                      AND (
                          s.created_at > workspaces.size_last_calculated_at
                          OR s.updated_at > workspaces.size_last_calculated_at
                          OR s.size_last_calculated_at > workspaces.size_last_calculated_at
                      )
                )
        "#
    )
    .execute(pool)
    .await
    .map_err(map_db)?;
    Ok(result.rows_affected())
}
