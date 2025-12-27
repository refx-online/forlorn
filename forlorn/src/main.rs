mod config;
mod constants;
mod dto;
mod geoloc;
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
use infrastructure::{SubscriberHandler, database, datadog, redis};
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
    let (redis_conn, subscriber_conn, score_locks) =
        redis::create_connection(&config.redis).await?;
    let metrics = Arc::new(datadog::create_metric(config.datadog.clone()));

    let storage = Storage::new(
        config.omajinai.beatmap_path.clone(),
        config.replay_path.clone(),
        config.screenshot_path.clone(),
        config.osz_path.clone(),
        &config.r2.bucket,
        &format!("https://{}.r2.cloudflarestorage.com", config.r2.account_id),
        &config.r2.access_key,
        &config.r2.secret_key,
    )
    .await;

    let state = AppState::new(
        config.clone(),
        storage,
        db_pool,
        redis_conn,
        subscriber_conn,
        score_locks,
        metrics,
    );

    let subscriber = SubscriberHandler::new(state.clone());

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
