use anyhow::Context;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::time::Duration;

#[allow(async_fn_in_trait)]
pub trait KvStore: Serialize + DeserializeOwned {
    const EXPIRE_IN: usize;

    fn key_format(key: String) -> String;

    fn expire_unix() -> i64 {
        chrono::Utc::now().timestamp() + Self::EXPIRE_IN as i64
    }

    async fn set_ex(&self, pool: &SqlitePool, key: String) -> anyhow::Result<()> {
        let value = serde_json::to_string(self).context("failed to serialize kv_store value")?;

        sqlx::query!(
            r#"
                INSERT INTO kv_store ("key", "value", expires)
                VALUES ($1, $2, $3)
                ON CONFLICT ("key") DO UPDATE
                SET "value" = excluded.value, expires = excluded.expires
            "#,
            Self::key_format(key),
            value,
            Self::expire_unix(),
        )
        .execute(pool)
        .await
        .context("failed to set value in kv_store")?;

        Ok(())
    }

    async fn get(pool: &SqlitePool, key: String) -> anyhow::Result<Self> {
        Self::try_get(pool, key)
            .await?
            .context("failed to get value from kv_store")
    }

    async fn try_get(pool: &SqlitePool, key: String) -> anyhow::Result<Option<Self>> {
        let value = sqlx::query_scalar!(
            r#"
                SELECT "value"
                FROM kv_store
                WHERE "key" = $1 AND expires > unixepoch()
            "#,
            Self::key_format(key),
        )
        .fetch_optional(pool)
        .await
        .context("failed to get value from kv_store")?;

        value
            .map(|value| {
                serde_json::from_str(&value).context("failed to deserialize kv_store value")
            })
            .transpose()
    }

    async fn get_ex(pool: &SqlitePool, key: String) -> anyhow::Result<Self> {
        Self::try_get_ex(pool, key)
            .await?
            .context("failed to get value from kv_store")
    }

    async fn try_get_ex(pool: &SqlitePool, key: String) -> anyhow::Result<Option<Self>> {
        let value = sqlx::query_scalar!(
            r#"UPDATE kv_store SET expires = $2
            WHERE "key" = $1 AND expires > unixepoch()
            RETURNING "value""#,
            Self::key_format(key),
            Self::expire_unix(),
        )
        .fetch_optional(pool)
        .await
        .context("failed to get value from kv_store")?;

        value
            .map(|value| {
                serde_json::from_str(&value).context("failed to deserialize kv_store value")
            })
            .transpose()
    }

    async fn get_del(pool: &SqlitePool, key: String) -> anyhow::Result<Self> {
        let value = sqlx::query_scalar!(
            r#"
                DELETE FROM kv_store
                WHERE "key" = $1 AND expires > unixepoch()
                RETURNING "value"
            "#,
            Self::key_format(key),
        )
        .fetch_one(pool)
        .await
        .context("failed to get value from kv_store")?;

        serde_json::from_str(&value).context("failed to deserialize kv_store value")
    }

    async fn del(pool: &SqlitePool, key: String) -> anyhow::Result<()> {
        sqlx::query!(
            r#"DELETE FROM kv_store WHERE "key" = $1"#,
            Self::key_format(key),
        )
        .execute(pool)
        .await
        .context("failed to delete value from kv_store")?;

        Ok(())
    }
}

pub async fn kvstore_cleanup(pool: SqlitePool) {
    loop {
        if let Err(err) = sqlx::query!(r#"DELETE FROM kv_store WHERE expires < unixepoch()"#)
            .execute(&pool)
            .await
        {
            tracing::warn!(error = %err, "kv_store cleanup failed");
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
