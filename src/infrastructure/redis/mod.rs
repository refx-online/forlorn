use anyhow::Result;
use redis::Client;
use redis::aio::MultiplexedConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::RedisConfig;

pub type RedisConnectionManager = Arc<Mutex<MultiplexedConnection>>;

pub async fn create_connection(config: &RedisConfig) -> Result<RedisConnectionManager> {
    let redis_url = match &config.password {
        Some(pass) => format!(
            "redis://:{}@{}:{}/{}",
            pass, config.host, config.port, config.db
        ),
        None => format!("redis://{}:{}/{}", config.host, config.port, config.db),
    };

    let client = Client::open(redis_url)?;
    let connection = client.get_multiplexed_async_connection().await?;

    Ok(Arc::new(Mutex::new(connection)))
}
