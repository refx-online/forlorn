use axum::{Json, http::StatusCode};
use serde_json::{Value, json};

pub async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}
