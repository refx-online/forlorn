pub mod beatmap;
pub mod channel;
pub mod connection;
pub mod direct;
pub mod error;
pub mod essentials;
pub mod favourite;
pub mod lastfm;
pub mod leaderboard;
pub mod rating;
pub mod replay;
pub mod screenshot;
pub mod submission;
pub mod v1;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

/// Unhandled routes:
/// - /web/osu-comment.php - i fucking hate this
/// - /web/osu-session.php - only for profiling and logging purposes, i don't think
///                          i should implement this?
/// - /users/ - no
/// - every beatmap submission related - i will separate it
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", v1::create_routes())
        // osu route
        .route(
            "/web/osu-submit-modular-selector.php",
            post(submission::submit_score),
        )
        .route(
            "/web/bancho_osu_connect.php",
            get(connection::get_bancho_connect),
        )
        .route(
            "/web/osu-screenshot.php",
            post(screenshot::upload_screenshot),
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
        .route("/web/lastfm.php", get(lastfm::get_lastfm))
        .route("/web/osu-getfriends.php", get(essentials::get_friends))
        .route("/web/osu-markasread.php", get(channel::mark_as_read))
        .route("/web/osu-search.php", get(direct::get_direct_search))
        .route(
            "/web/osu-search-set.php",
            get(direct::get_direct_search_set),
        )
        .route("/web/osu-error.php", post(error::get_error))
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
            get(connection::get_bancho_connect),
        )
        .route(
            "/web/refx-screenshot.php",
            post(screenshot::upload_screenshot),
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
        .route("/web/firstam.php", get(lastfm::get_lastfm))
        .route("/web/refx-getfriends.php", get(essentials::get_friends))
        .route("/web/refx-markasread.php", get(channel::mark_as_read))
        .route("/web/refx-search.php", get(direct::get_direct_search))
        .route(
            "/web/refx-search-set.php",
            get(direct::get_direct_search_set),
        )
        // essentials
        .route("/web/maps/{filename}", get(essentials::get_updated_beatmap))
        .route("/web/check-updates.php", get(essentials::get_check_updates))
        .route("/p/doyoureallywanttoaskpeppy", get(essentials::get_peppy))
        .route("/ss/{filename}", get(screenshot::get_screenshot))
        .route("/d/{mapset_id}", get(essentials::get_osz))
        .route(
            "/difficulty-rating",
            post(essentials::post_difficulty_rating),
        )
        .route("/beatmaps/{map_id}", get(essentials::get_redirect_beatmap))
        .route("/u/{user_id}", get(essentials::get_redirect_profile))
        .route("/users/{user_id}", get(essentials::get_redirect_profile))
}
