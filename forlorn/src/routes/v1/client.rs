use axum::{Json, http::StatusCode};
use serde_json::{Value, json};

use crate::constants::REFX_CURRENT_CLIENT_HASH;

pub async fn get_client() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "md5": REFX_CURRENT_CLIENT_HASH,
            "version": "20251108.1",
        })),
    )
}
