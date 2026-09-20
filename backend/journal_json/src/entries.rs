use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use db::AppState;
use errors::AppError;
use journal::entries::{self, Entry, EntryPage, ListOrder};
use serde::Deserialize;
use utils::UserId;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/notebooks/{notebook_id}/entries", get(list).post(create))
        .route("/entries/{id}", get(get_one).patch(update).delete(delete))
}

#[derive(Deserialize)]
struct CreateEntry {
    #[serde(with = "journal::b64")]
    title: Vec<u8>,
    #[serde(with = "journal::b64")]
    content: Vec<u8>,
    #[serde(default)]
    entry_date: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct ListQuery {
    cursor: Option<Uuid>,
    #[serde(default)]
    order: ListOrder,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct UpdateEntry {
    #[serde(default, deserialize_with = "journal::b64_opt::deserialize")]
    title: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "journal::b64_opt::deserialize")]
    content: Option<Vec<u8>>,
    #[serde(default)]
    entry_date: Option<NaiveDate>,
}

async fn list(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(notebook_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<EntryPage>, AppError> {
    let limit = entries::list_limit(query.limit)?;
    Ok(Json(
        entries::list(
            &state.db,
            user_id,
            notebook_id,
            query.cursor,
            query.order,
            limit,
        )
        .await?,
    ))
}

async fn get_one(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<Entry>, AppError> {
    Ok(Json(entries::get(&state.db, user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(notebook_id): Path<Uuid>,
    Json(body): Json<CreateEntry>,
) -> Result<(StatusCode, Json<Entry>), AppError> {
    let entry = entries::create(
        &state.db,
        user_id,
        notebook_id,
        &body.title,
        &body.content,
        body.entry_date,
    )
    .await?;
    tracing::info!(
        entry_id = %entry.id,
        notebook_id = %notebook_id,
        user_id = %user_id,
        "entry created"
    );
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn update(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEntry>,
) -> Result<StatusCode, AppError> {
    entries::update(
        &state.db,
        user_id,
        id,
        body.title.as_deref(),
        body.content.as_deref(),
        body.entry_date,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    entries::delete(&state.db, user_id, id).await?;
    tracing::info!(entry_id = %id, user_id = %user_id, "entry deleted");
    Ok(StatusCode::NO_CONTENT)
}
