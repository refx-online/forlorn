use crate::infrastructure::redis::RedisConnectionManager;
use super::publish;

pub async fn announce(redis: &RedisConnectionManager, message: &str) -> anyhow::Result<()> {
    publish(redis, "refx:announce", message).await?;

    Ok(())
}
