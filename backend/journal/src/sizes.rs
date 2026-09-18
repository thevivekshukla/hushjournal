use errors::AppError;
use sqlx::PgPool;

use crate::map_db;

pub async fn recalculate_stale_notebook_sizes(pool: &PgPool) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
            UPDATE notebooks s
            SET total_notebook_size = (
                    SELECT COALESCE(SUM(e.total_size), 0)::bigint
                    FROM entries e
                    WHERE e.notebook_id = s.id
                ),
                size_last_calculated_at = now()
            WHERE s.size_last_calculated_at IS NULL
               OR EXISTS (
                    SELECT 1
                    FROM entries e
                    WHERE e.notebook_id = s.id
                      AND (
                          e.created_at > s.size_last_calculated_at
                          OR e.updated_at > s.size_last_calculated_at
                      )
                )
        "#
    )
    .execute(pool)
    .await
    .map_err(map_db)?;
    Ok(result.rows_affected())
}

pub async fn recalculate_stale_journal_sizes(pool: &PgPool) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
            UPDATE journals w
            SET total_journal_size = (
                    SELECT COALESCE(SUM(s.total_notebook_size), 0)::bigint
                    FROM notebooks s
                    WHERE s.journal_id = w.id
                ),
                size_last_calculated_at = now()
            WHERE w.size_last_calculated_at IS NULL
               OR EXISTS (
                    SELECT 1
                    FROM notebooks s
                    WHERE s.journal_id = w.id
                      AND (
                          s.created_at > w.size_last_calculated_at
                          OR s.updated_at > w.size_last_calculated_at
                          OR s.size_last_calculated_at > w.size_last_calculated_at
                      )
                )
        "#
    )
    .execute(pool)
    .await
    .map_err(map_db)?;
    Ok(result.rows_affected())
}
