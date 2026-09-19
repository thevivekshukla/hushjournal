use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use db::AppState;
use errors::AppError;
use journal::journals::{self, Journal};
use serde::Deserialize;
use utils::UserId;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/journals", get(list).post(create))
        .route("/journals/{id}", get(get_one).patch(update).delete(delete))
}

#[derive(Deserialize)]
struct CreateJournal {
    name: String,
    #[serde(with = "journal::b64")]
    key_salt: Vec<u8>,
    #[serde(with = "journal::b64")]
    encrypted_dek: Vec<u8>,
    passphrase_hint: Option<String>,
    theme: Option<String>,
}

#[derive(Deserialize)]
struct UpdateJournal {
    name: Option<String>,
    #[serde(default, deserialize_with = "journal::b64_opt::deserialize")]
    key_salt: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "journal::b64_opt::deserialize")]
    encrypted_dek: Option<Vec<u8>>,
    passphrase_hint: Option<String>,
    mask: Option<bool>,
    theme: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<Vec<Journal>>, AppError> {
    Ok(Json(journals::list(&state.db, user_id).await?))
}

async fn get_one(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<Json<Journal>, AppError> {
    Ok(Json(journals::get(&state.db, user_id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<CreateJournal>,
) -> Result<(StatusCode, Json<Journal>), AppError> {
    let journal = journals::create(
        &state.db,
        user_id,
        &body.name,
        &body.key_salt,
        &body.encrypted_dek,
        body.passphrase_hint.as_deref(),
        body.theme.as_deref(),
    )
    .await?;
    tracing::info!(journal_id = %journal.id, user_id = %user_id, "journal created");
    Ok((StatusCode::CREATED, Json(journal)))
}

async fn update(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateJournal>,
) -> Result<Json<Journal>, AppError> {
    Ok(Json(
        journals::update(
            &state.db,
            user_id,
            id,
            body.name.as_deref(),
            body.key_salt.as_deref(),
            body.encrypted_dek.as_deref(),
            body.passphrase_hint.as_deref(),
            body.mask,
            body.theme.as_deref(),
        )
        .await?,
    ))
}

async fn delete(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    journals::delete(&state.db, user_id, id).await?;
    tracing::info!(journal_id = %id, user_id = %user_id, "journal deleted");
    Ok(StatusCode::NO_CONTENT)
}
