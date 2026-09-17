use anyhow::Context;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use db::AppState;
use errors::AppError;
use serde::Serialize;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utils::Config;

mod spa;

#[derive(Parser)]
#[command(name = "e2ejournal", about = "End-to-end encrypted journal API")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Backup the SQLite database
    #[command(name = "db-backup")]
    DbBackup {
        /// File to write the backup to
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    match Cli::parse().command {
        Some(Command::DbBackup { path }) => db_backup(path).await,
        None => serve().await,
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn db_backup(path: Option<PathBuf>) -> anyhow::Result<()> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set (see .env.example)")?;
    let src = db::sqlite_file_path(&database_url)?;
    let dest = path.unwrap_or_else(|| db::default_backup_path(&src));
    let dest = db::backup_to(&database_url, &dest).await?;
    tracing::info!("wrote backup to {}", dest.display());
    println!("{}", dest.display());
    Ok(())
}

async fn serve() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let state = AppState::connect(
        &config.database_url,
        config.cookie_secure,
        config.app_origin.clone(),
        config.google_oauth.clone(),
    )
    .await?;
    tokio::spawn(db::kvstore_cleanup(state.db.clone()));
    tokio::spawn(size_cron(state.db.clone()));

    let listener = TcpListener::bind(config.bind_addr())
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr()))?;

    tracing::info!("listening on {}", listener.local_addr()?);
    if spa::is_embedded() {
        tracing::info!("serving embedded SPA");
    } else {
        tracing::info!("no embedded SPA; API only (build with `just build` for production)");
    }

    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn size_cron(pool: sqlx::SqlitePool) {
    loop {
        match workspace::sizes::recalculate_stale_shelf_sizes(&pool).await {
            Ok(n) if n > 0 => tracing::info!(shelves = n, "recalculated shelf sizes"),
            Ok(_) => {}
            Err(err) => tracing::warn!(error = %err, "shelf size cron failed"),
        }
        match workspace::sizes::recalculate_stale_workspace_sizes(&pool).await {
            Ok(n) if n > 0 => tracing::info!(workspaces = n, "recalculated workspace sizes"),
            Ok(_) => {}
            Err(err) => tracing::warn!(error = %err, "workspace size cron failed"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api", user_json::router().merge(workspace_json::router()))
        .fallback(get(spa::fallback))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    sqlx::query_scalar!("SELECT 1").fetch_one(&state.db).await?;
    Ok(Json(HealthResponse { status: "ok" }))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
