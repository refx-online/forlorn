pub mod health;
pub mod submission;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route(
            "/web/osu-submit-modular-selector.php",
            post(submission::submit_score),
        )
        .route(
            "/web/refx-submit-modular.php",
            post(submission::submit_score),
        ) // todo: refactor client route
}
