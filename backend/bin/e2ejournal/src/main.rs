use anyhow::Context;
use axum::extract::State;
use axum::http::{Method, header};
use axum::routing::get;
use axum::{Json, Router};
use db::AppState;
use errors::AppError;
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utils::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let state = AppState::connect(
        &config.database_url,
        config.cookie_secure,
        config.google_oauth.clone(),
    )
    .await?;
    tokio::spawn(db::pgstore_cleanup(state.db.clone()));

    let listener = TcpListener::bind(config.bind_addr())
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr()))?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);

    Router::new()
        .route("/health", get(health))
        .nest("/api", user_json::router().merge(workspace_json::router()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
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
