mod google;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::CookieJar;
use db::AppState;
use errors::AppError;
use rand::RngCore;
use serde::Deserialize;
use user::User;
use utils::{Session, UserId};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/google", get(google_login))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/logout", post(logout))
        .route("/user", get(me).patch(update_me).delete(delete_me))
}

#[derive(Deserialize)]
struct GoogleLoginQuery {
    next: Option<String>,
}

#[derive(Deserialize)]
struct GoogleCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct UpdateProfile {
    name: String,
}

async fn google_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<GoogleLoginQuery>,
    mut session: Session,
) -> Result<impl IntoResponse, AppError> {
    let oauth_state = random_token();
    session
        .attach(&state.db, "oauth_state", &oauth_state)
        .await?;
    if let Some(next) = safe_next(query.next.as_deref()) {
        session.attach(&state.db, "oauth_next", &next).await?;
    } else {
        session.remove(&state.db, "oauth_next").await?;
    }

    let url = google::authorization_url(&state, &oauth_state);
    Ok((
        with_session_cookie(jar, &session, state.cookie_secure),
        Redirect::temporary(&url),
    ))
}

async fn google_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<GoogleCallbackQuery>,
    mut session: Session,
) -> Result<impl IntoResponse, AppError> {
    if query.error.as_deref() == Some("access_denied") {
        return Err(AppError::BadRequest("google login was cancelled".into()));
    }
    if query.error.is_some() {
        return Err(AppError::BadRequest("google login failed".into()));
    }

    let Some(code) = query.code.filter(|code| !code.is_empty()) else {
        return Err(AppError::BadRequest("missing oauth code".into()));
    };
    let Some(returned_state) = query.state.filter(|state| !state.is_empty()) else {
        return Err(AppError::BadRequest("missing oauth state".into()));
    };
    let Some(expected_state) = session.get::<String>("oauth_state") else {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    };
    if returned_state != expected_state {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    }

    let next = session
        .get::<String>("oauth_next")
        .and_then(|next| safe_next(Some(&next)));
    session.remove(&state.db, "oauth_state").await?;
    session.remove(&state.db, "oauth_next").await?;

    let account = google::exchange_code(&state, &code).await?;
    let user = user::login_with_google(&state.db, &account).await?;
    session.attach(&state.db, "user_id", &user.id).await?;
    tracing::info!(user_id = %user.id, "user logged in with google");

    let jar = with_session_cookie(jar, &session, state.cookie_secure);
    if let Some(next) = next {
        Ok((jar, Redirect::to(&next)).into_response())
    } else {
        Ok((jar, Json(user)).into_response())
    }
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    mut session: Session,
) -> Result<impl IntoResponse, AppError> {
    session.destroy(&state.db).await?;
    Ok((
        jar.add(Session::removal_cookie(state.cookie_secure)),
        StatusCode::NO_CONTENT,
    ))
}

async fn me(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Result<Json<User>, AppError> {
    Ok(Json(user::get_by_id(&state.db, user_id).await?))
}

async fn update_me(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<UpdateProfile>,
) -> Result<Json<User>, AppError> {
    Ok(Json(
        user::update_name(&state.db, user_id, &body.name).await?,
    ))
}

async fn delete_me(
    State(state): State<AppState>,
    jar: CookieJar,
    mut session: Session,
    UserId(user_id): UserId,
) -> Result<impl IntoResponse, AppError> {
    user::delete_by_id(&state.db, user_id).await?;
    session.destroy(&state.db).await?;
    Ok((
        jar.add(Session::removal_cookie(state.cookie_secure)),
        StatusCode::NO_CONTENT,
    ))
}

fn with_session_cookie(jar: CookieJar, session: &Session, secure: bool) -> CookieJar {
    jar.add(session.cookie(secure))
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn safe_next(next: Option<&str>) -> Option<String> {
    let next = next?.trim();
    if next.starts_with('/') && !next.starts_with("//") && !next.contains('\\') {
        Some(next.to_string())
    } else {
        None
    }
}
