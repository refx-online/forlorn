mod config;
mod constants;
mod dto;
mod infrastructure;
mod models;
mod repository;
mod routes;
mod state;
mod usecases;
mod utils;

use std::sync::Arc;

use anyhow::Result;
use config::Config;
use dotenvy::dotenv;
use infrastructure::{database, redis};
use routes::create_routes;
use state::AppState;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let config = Config::from_env()?;
    let config = Arc::new(config);

    let db_pool = database::create_pool(&config.database).await?;
    let redis_conn = redis::create_connection(&config.redis).await?;

    let state = AppState::new(config.clone(), db_pool, redis_conn);

    let app = create_routes().with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("forlorn running on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
