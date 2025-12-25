use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use webhook::Webhook;

use crate::{
    constants::LastFmFlags, dto::lastfm::GetLastFm, infrastructure::redis::publish::restrict,
    models::User, repository, state::AppState, usecases::password::verify_password,
};

async fn authenticate_user(
    state: &AppState,
    password_md5: &str,
    username: &str,
) -> Result<User, Response> {
    let user = match repository::user::fetch_by_name(&state.db, username).await {
        Ok(Some(user)) => user,
        _ => {
            return Err(StatusCode::OK.into_response());
        },
    };

    match verify_password(password_md5, &user.pw_bcrypt).await {
        Ok(true) => Ok(user),
        _ => Err(StatusCode::OK.into_response()),
    }
}

pub async fn get_lastfm(
    State(state): State<AppState>,
    Query(lastfm): Query<GetLastFm>,
) -> impl IntoResponse {
    if !lastfm.flag.starts_with('a') {
        return (StatusCode::OK, b"-3").into_response();
    }

    let raw = match lastfm.flag[1..].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return (StatusCode::OK, b"-3").into_response(),
    };

    let user = match authenticate_user(&state, &lastfm.password_md5, &lastfm.username).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    let flags = LastFmFlags::from_bits_truncate(raw);
    let explanations = flags.explain().join("\n");

    // would be funny if someone still has hq!osu
    if flags.contains(LastFmFlags::HQ_ASSEMBLY) || flags.contains(LastFmFlags::HQ_FILE) {
        tokio::spawn(async move {
            let _ = restrict::restrict(
                &state.redis,
                user.id,
                &format!("hq!osu files found ({})", explanations),
            )
            .await;
        });
        return (StatusCode::OK, b"-3").into_response();
    }

    // they're probably already patched `ConfigManager`
    // or some weak edits using `Harmony` or `Cheat Engine`
    // yet, it's still punishable. since they are modifying the client.
    if flags.contains(LastFmFlags::INVALID_CHEAT_VALUES) {
        tokio::spawn(async move {
            let _ = restrict::restrict(
                &state.redis,
                user.id,
                &format!("invalid cheat values ({})", explanations),
            )
            .await;
        });
        return (StatusCode::OK, b"-3").into_response();
    }

    // not sure if we want to restrict for these
    // since its possible that they doesn't even remember those multi account times
    // if flags.contains(LastFmFlags::REGISTRY_EDITS) {
    //     tokio::spawn(async move {
    //         let _ = restrict::restrict(&state.redis, user.id, "hq!osu relife registry edits found").await;
    //     });
    //     return (StatusCode::OK, b"-3").into_response();
    // }

    let webhook = Webhook::new(&state.config.webhook.debug).content(format!(
        "{} has been flagged with: 0x{} ({})",
        user.name(),
        raw,
        explanations
    ));

    tracing::warn!(
        "{} has been flagged with: 0x{} ({})",
        user.name(),
        raw,
        explanations
    );

    let _ = state
        .metrics
        .incr("lastfm.flagged", [format!("flag:{raw}")]);

    tokio::spawn(async move {
        let _ = webhook.post().await;
    });

    (StatusCode::OK, b"-3").into_response()
}
