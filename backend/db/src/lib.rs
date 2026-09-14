use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

mod pg_store;
mod session;

pub use pg_store::{PgStore, pgstore_cleanup};
pub use session::{SESSION_COOKIE, Session};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cookie_secure: bool,
}

impl AppState {
    pub async fn connect(database_url: &str, cookie_secure: bool) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("failed to connect to postgres")?;

        sqlx::migrate!()
            .run(&db)
            .await
            .context("failed to run migrations")?;

        Ok(Self { db, cookie_secure })
    }
}
