use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{json, Value};

use crate::{
    constants::RankedStatus,
    infrastructure::redis::publish,
    state::AppState,
};

// discord interactions endpoint for staff commands (currently just /rank).
// setup is dashboard-side: point discord's interactions url at
// <forlorn>/discord/interactions and register the /rank command once
// (see .env.example). no gateway connection needed.
pub async fn interactions(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let cfg = &state.config.discord_interactions;
    if cfg.public_key.is_empty() || cfg.staff_role_ids.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "discord commands disabled")
            .into_response();
    }

    let Some(sig) = headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
    else {
        return (StatusCode::UNAUTHORIZED, "missing signature").into_response();
    };
    let Some(timestamp) = headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
    else {
        return (StatusCode::UNAUTHORIZED, "missing timestamp").into_response();
    };

    if !verify(&cfg.public_key, sig, timestamp, &body) {
        return (StatusCode::UNAUTHORIZED, "bad signature").into_response();
    }

    let value: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "bad json").into_response(),
    };

    match value.get("type").and_then(Value::as_u64) {
        // discord ping — must answer with a pong
        Some(1) => Json(json!({"type": 1})).into_response(),
        Some(2) => handle_command(&state, &value).await.into_response(),
        _ => (StatusCode::BAD_REQUEST, "unknown type").into_response(),
    }
}

fn verify(public_key_hex: &str, sig_hex: &str, timestamp: &str, body: &[u8]) -> bool {
    let Ok(pk_bytes) = hex::decode(public_key_hex) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(sig_hex) else {
        return false;
    };
    let Ok(pk_arr) = <[u8; 32]>::try_from(pk_bytes) else {
        return false;
    };
    let Ok(sig_arr) = <[u8; 64]>::try_from(sig_bytes) else {
        return false;
    };

    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    VerifyingKey::from_bytes(&pk_arr)
        .and_then(|key| {
            Signature::from_slice(&sig_arr).map(|sig| (key, sig))
        })
        .and_then(|(key, sig)| key.verify(&message, &sig))
        .is_ok()
}

fn ephemeral(content: String) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "type": 4,
            "data": {"content": content, "flags": 64},
        })),
    )
}

fn opt_str<'a>(options: &'a [Value], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find(|o| o.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|o| o.get("value"))
        .and_then(Value::as_str)
}

fn opt_int(options: &[Value], name: &str) -> Option<i32> {
    options
        .iter()
        .find(|o| o.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|o| o.get("value"))
        .and_then(Value::as_i64)
        .and_then(|v| v.try_into().ok())
}

async fn handle_command(state: &AppState, value: &Value) -> (StatusCode, Json<Value>) {
    if value
        .get("data")
        .and_then(|d| d.get("name"))
        .and_then(Value::as_str)
        != Some("rank")
    {
        return ephemeral("unknown command".into());
    }

    // staff gate: caller needs one of the configured role ids
    let roles: Vec<&str> = value
        .get("member")
        .and_then(|m| m.get("roles"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if !roles
        .iter()
        .any(|r| state.config.discord_interactions.staff_role_ids.iter().any(|s| s == r))
    {
        return ephemeral("no permission".into());
    }

    let options: Vec<Value> = value
        .get("data")
        .and_then(|d| d.get("options"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let status = match opt_str(&options, "status") {
        Some("rank") => RankedStatus::Ranked,
        Some("unrank") => RankedStatus::Pending,
        Some("love") => RankedStatus::Loved,
        Some("qual") => RankedStatus::Qualified,
        _ => return ephemeral("status must be rank/unrank/love/qual".into()),
    };
    let scope = match opt_str(&options, "scope") {
        Some("map") | None => "map",
        Some("set") => "set",
        _ => return ephemeral("scope must be map/set".into()),
    };
    let Some(target) = opt_int(&options, "target") else {
        return ephemeral("target must be a map (or set) id".into());
    };

    // same semantics as the ingame ?map command: flip status+frozen, then
    // tell the world to refresh the map(s).
    let rows: Vec<(i32, i32, String, String, String, String, i32)> = match sqlx::query_as(
        "SELECT id, set_id, md5, artist, title, version, status FROM maps WHERE id = ? OR set_id = ?",
    )
    .bind(target)
    .bind(target)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("discord rank: db lookup failed: {e:?}");
            return ephemeral("database error".into());
        },
    };

    let maps: Vec<_> = rows
        .into_iter()
        .filter(|(id, set_id, ..)| {
            if scope == "map" {
                *id == target
            } else {
                *set_id == target
            }
        })
        .collect();

    if maps.is_empty() {
        return ephemeral("couldn't find that map in the database.".into());
    }

    let label = match scope {
        "map" => {
            let (id, _, _, artist, title, version, _) = &maps[0];
            if let Err(e) = sqlx::query("UPDATE maps SET status = ?, frozen = 1 WHERE id = ?")
                .bind(status.as_i32())
                .bind(id)
                .execute(state.db.as_ref())
                .await
            {
                tracing::warn!("discord rank: update failed: {e:?}");
                return ephemeral("database error".into());
            }
            format!("{artist} - {title} [{version}]")
        },
        _ => {
            if let Err(e) =
                sqlx::query("UPDATE maps SET status = ?, frozen = 1 WHERE set_id = ?")
                    .bind(status.as_i32())
                    .bind(target)
                    .execute(state.db.as_ref())
                    .await
            {
                tracing::warn!("discord rank: update failed: {e:?}");
                return ephemeral("database error".into());
            }
            format!("set {target} ({} maps)", maps.len())
        },
    };

    for (_, _, md5, ..) in &maps {
        if let Err(e) = publish::publish(&state.redis, "forlorn:refresh_map", md5).await {
            tracing::warn!("discord rank: refresh publish failed: {e:?}");
        }
    }

    let status_name = match status {
        RankedStatus::Ranked => "ranked",
        RankedStatus::Pending => "unranked",
        RankedStatus::Loved => "loved",
        RankedStatus::Qualified => "qualified",
        _ => "updated",
    };
    ephemeral(format!("{label} has been {status_name}."))
}
