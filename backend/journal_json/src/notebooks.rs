use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use db::AppState;
use errors::AppError;
use journal::notebooks::{self, Notebook};
use serde::Deserialize;
use utils::UserId;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/journals/{journal_id}/notebooks", get(list).post(create))
        .route("/notebooks/{id}", get(get_one).patch(update).delete(delete))
}

#[derive(Deserialize)]
struct CreateNotebook {
    #[serde(with = "journal::b64")]
    name: Vec<u8>,
    icon: Option<String>,
}

#[derive(Deserialize)]
struct UpdateNotebook {
    #[serde(default, deserialize_with = "journal::b64_opt::deserialize")]
    name: Option<Vec<u8>>,
    #[serde(default)]
    icon: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(journal_id): Path<Uuid>,
) -> Result<Json<Vec<Notebook>>, AppError> {
    Ok(Json(notebooks::list(&state.db, user_id, journal_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<Notebook>, AppError> {
    Ok(Json(notebooks::get(&state.db, user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(journal_id): Path<Uuid>,
    Json(body): Json<CreateNotebook>,
) -> Result<(StatusCode, Json<Notebook>), AppError> {
    let notebook = notebooks::create(
        &state.db,
        user_id,
        journal_id,
        &body.name,
        body.icon.as_deref(),
    )
    .await?;
    tracing::info!(
        notebook_id = %notebook.id,
        journal_id = %journal_id,
        user_id = %user_id,
        "notebook created"
    );
    Ok((StatusCode::CREATED, Json(notebook)))
}

async fn update(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateNotebook>,
) -> Result<Json<Notebook>, AppError> {
    let icon_set = body.icon.is_some();
    Ok(Json(
        notebooks::update(
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
    notebooks::delete(&state.db, user_id, id).await?;
    tracing::info!(notebook_id = %id, user_id = %user_id, "notebook deleted");
    Ok(StatusCode::NO_CONTENT)
}
