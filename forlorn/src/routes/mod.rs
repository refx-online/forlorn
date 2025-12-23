pub mod beatmap;
pub mod essentials;
pub mod favourite;
pub mod health;
pub mod leaderboard;
pub mod rating;
pub mod replay;
pub mod screenshot;
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
            "/web/osu-submit-modular-selector.php",
            post(submission::submit_score),
        )
        .route(
            "/web/bancho_osu_connect.php",
            get(essentials::get_bancho_connect),
        )
        .route(
            "/web/osu-screenshot.php",
            get(screenshot::upload_screenshot),
        )
        .route(
            "/web/osu-getbeatmapinfo.php",
            get(beatmap::get_beatmap_info),
        )
        .route("/web/osu-osz2-getscores.php", get(leaderboard::get_scores))
        .route("/web/osu-getreplay.php", get(replay::get_replay))
        .route("/web/osu-rate.php", get(rating::get_rating))
        .route("/web/osu-getfavourites.php", get(favourite::get_favourites))
        .route("/web/osu-addfavourite.php", get(favourite::add_favourites))
        // refx route
        // TODO: ask myself in the future to revert these ancient routes
        //       to its original route, so i dont have to
        //       write these shit all over again.
        .route(
            "/web/refx-submit-modular.php",
            post(submission::submit_score),
        )
        .route(
            "/web/bancho_refx_connect.php",
            get(essentials::get_bancho_connect),
        )
        .route(
            "/web/refx-screenshot.php",
            get(screenshot::upload_screenshot),
        )
        .route(
            "/web/refx-getbeatmapinfo.php",
            get(beatmap::get_beatmap_info),
        )
        .route("/web/refx-osz2-getscores.php", get(leaderboard::get_scores))
        .route("/web/refx-getreplay.php", get(replay::get_replay))
        .route("/web/refx-rate.php", get(rating::get_rating))
        .route(
            "/web/refx-getfavourites.php",
            get(favourite::get_favourites),
        )
        .route("/web/refx-addfavourite.php", get(favourite::add_favourites))
        // essentials
        .route("/web/maps/{filename}", get(essentials::get_updated_beatmap))
        .route("/web/check-updates.php", get(essentials::get_check_updates))
        .route("/p/doyoureallywanttoaskpeppy", get(essentials::get_peppy))
        .route(
            "/ss/{screenshot_id}.{extension}",
            get(screenshot::get_screenshot),
        )
}
