use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use db::AppState;
use errors::AppError;
use serde::Deserialize;
use utils::UserId;
use uuid::Uuid;
use workspace::shelves::{self, Shelf};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workspaces/{workspace_id}/shelves", get(list).post(create))
        .route("/shelves/{id}", get(get_one).patch(update).delete(delete))
}

#[derive(Deserialize)]
struct CreateShelf {
    #[serde(with = "workspace::b64")]
    name: Vec<u8>,
    icon: Option<String>,
}

#[derive(Deserialize)]
struct UpdateShelf {
    #[serde(default, deserialize_with = "workspace::b64_opt::deserialize")]
    name: Option<Vec<u8>>,
    #[serde(default)]
    icon: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<Shelf>>, AppError> {
    Ok(Json(shelves::list(&state.db, user_id, workspace_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<Shelf>, AppError> {
    Ok(Json(shelves::get(&state.db, user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(workspace_id): Path<Uuid>,
    Json(body): Json<CreateShelf>,
) -> Result<(StatusCode, Json<Shelf>), AppError> {
    let shelf = shelves::create(
        &state.db,
        user_id,
        workspace_id,
        &body.name,
        body.icon.as_deref(),
    )
    .await?;
    tracing::info!(
        shelf_id = %shelf.id,
        workspace_id = %workspace_id,
        user_id = %user_id,
        "shelf created"
    );
    Ok((StatusCode::CREATED, Json(shelf)))
}

async fn update(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateShelf>,
) -> Result<Json<Shelf>, AppError> {
    let icon_set = body.icon.is_some();
    Ok(Json(
        shelves::update(
            &state.db,
            user_id,
            id,
            body.name.as_deref(),
            icon_set,
            body.icon.as_deref(),
        )
        .await?,
    ))
}

async fn delete(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    shelves::delete(&state.db, user_id, id).await?;
    tracing::info!(shelf_id = %id, user_id = %user_id, "shelf deleted");
    Ok(StatusCode::NO_CONTENT)
}
