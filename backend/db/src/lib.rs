use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

mod pg_store;
mod session;

pub use pg_store::{PgStore, pgstore_cleanup};
pub use session::{SESSION_COOKIE, Session};

#[derive(Clone, Debug)]
pub struct GoogleOAuth {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cookie_secure: bool,
    pub google_oauth: GoogleOAuth,
}

impl AppState {
    pub async fn connect(
        database_url: &str,
        cookie_secure: bool,
        google_oauth: GoogleOAuth,
    ) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("failed to connect to postgres")?;

        sqlx::migrate!()
            .run(&db)
            .await
            .context("failed to run migrations")?;

        Ok(Self {
            db,
            cookie_secure,
            google_oauth,
        })
    }
}
