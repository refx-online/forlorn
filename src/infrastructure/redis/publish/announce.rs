use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn announce(redis: &RedisConnectionManager, message: &str) -> anyhow::Result<()> {
    publish(redis, "refx:announce", message).await?;

    Ok(())
}
