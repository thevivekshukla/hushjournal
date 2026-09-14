use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("failed to connect to postgres")?;

        sqlx::migrate!()
            .run(&db)
            .await
            .context("failed to run migrations")?;

        Ok(Self { db })
    }
}
