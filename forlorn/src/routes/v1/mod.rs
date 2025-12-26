pub mod calculate;
pub mod client;
pub mod health;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/calculate", get(calculate::get_calculate_map))
        .route("/latest_refx_client_hash", get(client::get_client))
}
