use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use db::AppState;
use errors::AppError;
use serde::Deserialize;
use utils::UserId;
use uuid::Uuid;
use workspace::workspaces::{self, Workspace};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workspaces", get(list).post(create))
        .route(
            "/workspaces/{id}",
            get(get_one).patch(update).delete(delete),
        )
}

#[derive(Deserialize)]
struct CreateWorkspace {
    name: String,
    #[serde(with = "workspace::b64")]
    key_salt: Vec<u8>,
    #[serde(with = "workspace::b64")]
    encrypted_dek: Vec<u8>,
}

#[derive(Deserialize)]
struct UpdateWorkspace {
    name: Option<String>,
    #[serde(default, deserialize_with = "workspace::b64_opt::deserialize")]
    key_salt: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "workspace::b64_opt::deserialize")]
    encrypted_dek: Option<Vec<u8>>,
}

async fn list(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<Vec<Workspace>>, AppError> {
    Ok(Json(workspaces::list(&state.db, user_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<Workspace>, AppError> {
    Ok(Json(workspaces::get(&state.db, user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<CreateWorkspace>,
) -> Result<(StatusCode, Json<Workspace>), AppError> {
    let workspace = workspaces::create(
        &state.db,
        user_id,
        &body.name,
        &body.key_salt,
        &body.encrypted_dek,
    )
    .await?;
    tracing::info!(workspace_id = %workspace.id, user_id = %user_id, "workspace created");
    Ok((StatusCode::CREATED, Json(workspace)))
}

async fn update(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWorkspace>,
) -> Result<Json<Workspace>, AppError> {
    Ok(Json(
        workspaces::update(
            &state.db,
            user_id,
            id,
            body.name.as_deref(),
            body.key_salt.as_deref(),
            body.encrypted_dek.as_deref(),
        )
        .await?,
    ))
}

async fn delete(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    workspaces::delete(&state.db, user_id, id).await?;
    tracing::info!(workspace_id = %id, user_id = %user_id, "workspace deleted");
    Ok(StatusCode::NO_CONTENT)
}
