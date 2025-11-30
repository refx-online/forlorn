use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn announce(redis: &RedisConnectionManager, score: u64) -> anyhow::Result<()> {
    publish(redis, "refx:announce", &score.to_string()).await?;

    Ok(())
}
