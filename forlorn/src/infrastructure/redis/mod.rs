use std::sync::Arc;

use anyhow::Result;
use redis::{
    Client,
    aio::{MultiplexedConnection, PubSub},
};
use rslock::LockManager;
use tokio::sync::Mutex;

use crate::config::RedisConfig;

pub mod publish;
pub mod subscriber;

pub type RedisConnectionManager = Arc<Mutex<MultiplexedConnection>>;
pub type RedisPubsubManager = Arc<Mutex<PubSub>>;

pub async fn create_connection(
    config: &RedisConfig,
) -> Result<(RedisConnectionManager, RedisPubsubManager, LockManager)> {
    let redis_url = match &config.password {
        Some(pass) => format!(
            "redis://:{}@{}:{}/{}",
            pass, config.host, config.port, config.db
        ),
        None => format!("redis://{}:{}/{}", config.host, config.port, config.db),
    };

    let client = Client::open(redis_url.clone())?;

    let connection = client.get_multiplexed_async_connection().await?;
    let pubsub_connection = client.get_async_pubsub().await?;

    Ok((
        Arc::new(Mutex::new(connection)),
        Arc::new(Mutex::new(pubsub_connection)),
        LockManager::new(vec![redis_url]),
    ))
}
