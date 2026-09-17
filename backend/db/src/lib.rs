use anyhow::Context;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

mod kv_store;
mod session;

pub use kv_store::{KvStore, kvstore_cleanup};
pub use session::{SESSION_COOKIE, Session};

#[derive(Clone, Debug)]
pub struct GoogleOAuth {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub cookie_secure: bool,
    pub app_origin: Option<String>,
    pub google_oauth: GoogleOAuth,
}

impl AppState {
    pub async fn connect(
        database_url: &str,
        cookie_secure: bool,
        app_origin: Option<String>,
        google_oauth: GoogleOAuth,
    ) -> anyhow::Result<Self> {
        let db = connect_pool(database_url).await?;
        Ok(Self {
            db,
            cookie_secure,
            app_origin,
            google_oauth,
        })
    }
}

pub async fn connect_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    let mut options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("invalid DATABASE_URL: {database_url}"))?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    if !is_memory_url(database_url) {
        options = options
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        if let Some(parent) = sqlite_file_parent(database_url) {
            std::fs::create_dir_all(&parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }

    let max_connections = if is_memory_url(database_url) { 1 } else { 5 };
    let db = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await
        .context("failed to connect to sqlite")?;

    sqlx::migrate!()
        .run(&db)
        .await
        .context("failed to run migrations")?;

    Ok(db)
}

pub fn sqlite_file_path(database_url: &str) -> anyhow::Result<PathBuf> {
    if is_memory_url(database_url) {
        anyhow::bail!("in-memory sqlite databases cannot be backed up");
    }
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .context("DATABASE_URL must be a sqlite: file path")?;
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() || path.starts_with(':') {
        anyhow::bail!("DATABASE_URL has no sqlite file path");
    }
    Ok(Path::new(path).to_path_buf())
}

pub fn default_backup_path(src: &Path) -> PathBuf {
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let stem = src
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("e2ejournal");
    let filename = format!("{stem}-{stamp}.db");
    match src.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(filename),
        _ => PathBuf::from(filename),
    }
}

pub async fn backup_to(database_url: &str, dest: &Path) -> anyhow::Result<PathBuf> {
    let src = sqlite_file_path(database_url)?;
    if !src.exists() {
        anyhow::bail!("database file not found: {}", src.display());
    }

    let dest = if dest.is_absolute() {
        dest.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to resolve current directory")?
            .join(dest)
    };
    if dest.exists() {
        anyhow::bail!("backup path already exists: {}", dest.display());
    }
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let dest_str = dest
        .to_str()
        .with_context(|| format!("backup path is not valid UTF-8: {}", dest.display()))?;
    let pool = connect_pool(database_url).await?;
    sqlx::query!("VACUUM INTO $1", dest_str)
        .execute(&pool)
        .await
        .with_context(|| format!("failed to backup to {}", dest.display()))?;
    Ok(dest)
}

fn is_memory_url(database_url: &str) -> bool {
    database_url.contains(":memory:") || database_url.contains("mode=memory")
}

fn sqlite_file_parent(database_url: &str) -> Option<PathBuf> {
    let path = sqlite_file_path(database_url).ok()?;
    let parent = path.parent()?;
    if parent.as_os_str().is_empty() {
        None
    } else {
        Some(parent.to_path_buf())
    }
}
