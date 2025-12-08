pub mod health;
pub mod leaderboard;
pub mod replay;
pub mod submission;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        // osu route
        .route(
            "/web/maps/{filename}",
            get(leaderboard::get_updated_beatmap),
        )
        .route(
            "/web/osu-submit-modular-selector.php",
            post(submission::submit_score),
        )
        .route("/web/osu-osz2-getscores.php", get(leaderboard::get_scores))
        .route("/web/osu-getreplay.php", get(replay::get_replay))
        // refx route
        // TODO: ask myself in the future to revert these ancient routes
        //       to its original route, so i dont have to
        //       write these shit all over again.
        .route(
            "/web/refx-submit-modular.php",
            post(submission::submit_score),
        )
        .route("/web/refx-osz2-getscores.php", get(leaderboard::get_scores))
        .route("/web/refx-getreplay.php", get(replay::get_replay))
}
