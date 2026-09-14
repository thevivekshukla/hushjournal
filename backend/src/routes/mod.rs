use axum::routing::get;
use axum::Router;

use crate::state::AppState;

mod health;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health))
}
