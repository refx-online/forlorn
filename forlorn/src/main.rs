mod config;
mod constants;
mod dto;
mod infrastructure;
mod models;
mod repository;
mod routes;
mod state;
mod storage;
mod usecases;
mod utils;

use std::sync::Arc;

use anyhow::Result;
use config::Config;
use dotenvy::dotenv;
use infrastructure::{database, redis, redis::subscriber::SubscriberHandler, tasks};
use routes::create_routes;
use state::AppState;
use storage::Storage;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use utils::shutdown_signal;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let config = Config::from_env()?;
    let config = Arc::new(config);

    let db_pool = database::create_pool(&config.database).await?;
    let (redis_conn, subscriber_conn) = redis::create_connection(&config.redis).await?;

    let storage = Storage::new(
        config.omajinai.beatmap_path.clone(),
        config.replay_path.clone(),
        config.screenshot_path.clone(),
        config.osz_path.clone(),
    );

    let state = AppState::new(
        config.clone(),
        storage,
        db_pool,
        redis_conn,
        subscriber_conn,
    );

    let subscriber = SubscriberHandler::new(state.clone());

    tokio::spawn(tasks::cleanup_score_locks(state.score_locks.clone()));
    tokio::spawn(async move {
        if let Err(e) = subscriber.start_listener().await {
            tracing::error!("pubsub listener crashed: {e:?}");
        }
    });

    let app = create_routes().with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("running on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
