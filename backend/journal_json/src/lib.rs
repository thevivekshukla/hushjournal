mod entries;
mod journals;
mod notebooks;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use db::AppState;

const ENTRY_BODY_LIMIT: usize = 10 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(journals::router())
        .merge(notebooks::router())
        .merge(entries::router())
        .layer(DefaultBodyLimit::max(ENTRY_BODY_LIMIT))
}
