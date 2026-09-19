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
    pub disable_user_signup: bool,
    pub disable_password_form: bool,
    pub app_origin: Option<String>,
    pub google_oauth: GoogleOAuth,
}

impl AppState {
    pub async fn connect(
        database_url: &str,
        cookie_secure: bool,
        disable_user_signup: bool,
        disable_password_form: bool,
        app_origin: Option<String>,
        google_oauth: GoogleOAuth,
    ) -> anyhow::Result<Self> {
        let db = connect_pool(database_url).await?;
        Ok(Self {
            db,
            cookie_secure,
            disable_user_signup,
            disable_password_form,
            app_origin,
            google_oauth,
        })
    }
}

pub async fn connect_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("failed to connect to postgres")?;

    sqlx::migrate!()
        .run(&db)
        .await
        .context("failed to run migrations")?;

    Ok(db)
}

pub fn default_backup_path() -> std::path::PathBuf {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    std::path::PathBuf::from(format!("e2ejournal-{stamp}.dump"))
}

pub async fn backup_to(
    database_url: &str,
    dest: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    if dest.exists() {
        anyhow::bail!("backup path already exists: {}", dest.display());
    }
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let status = tokio::process::Command::new("pg_dump")
        .arg("--dbname")
        .arg(database_url)
        .arg("-Fc")
        .arg("-f")
        .arg(dest)
        .status()
        .await
        .context("failed to run pg_dump (is it installed?)")?;
    if !status.success() {
        anyhow::bail!("pg_dump failed with {status}");
    }
    Ok(dest.to_path_buf())
}
