use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use cookie::time::Duration as CookieDuration;
use db::{AppState, PgStore, Session as DbSession};
use errors::AppError;
use uuid::Uuid;

pub struct UserId(pub Uuid);

#[derive(Clone)]
pub struct Session(pub DbSession);

impl Session {
    pub fn cookie(&self, secure: bool) -> Cookie<'static> {
        Cookie::build((db::SESSION_COOKIE, self.0.raw_id().to_string()))
            .http_only(true)
            .path("/")
            .same_site(SameSite::Lax)
            .secure(secure)
            .max_age(CookieDuration::seconds(DbSession::EXPIRE_IN as i64))
            .build()
    }

    pub fn removal_cookie() -> Cookie<'static> {
        Cookie::build((db::SESSION_COOKIE, ""))
            .http_only(true)
            .path("/")
            .same_site(SameSite::Lax)
            .max_age(CookieDuration::ZERO)
            .build()
    }
}

impl std::ops::Deref for Session {
    type Target = DbSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRequestParts<AppState> for Session {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(session) = parts.extensions.get::<Session>().cloned() {
            return Ok(session);
        }

        let jar = CookieJar::from_headers(&parts.headers);
        let session = match jar.get(db::SESSION_COOKIE) {
            Some(cookie) => DbSession::load(&state.db, cookie.value())
                .await?
                .unwrap_or_else(DbSession::new),
            None => DbSession::new(),
        };
        let session = Session(session);
        parts.extensions.insert(session.clone());
        Ok(session)
    }
}

impl FromRequestParts<AppState> for UserId {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        session.user_id().map(UserId).ok_or(AppError::Unauthorized)
    }
}
