pub mod health;
pub mod leaderboard;
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
        .route("/web/osu-osz2-getscores.php", get(leaderboard::get_scores))
        .route("/web/refx-osz2-getscores.php", get(leaderboard::get_scores))
}
