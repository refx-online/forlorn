use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn score_submitted(redis: &RedisConnectionManager, score_id: u64) -> anyhow::Result<()> {
    publish(redis, "refx:score_submitted", &score_id.to_string()).await?;

    Ok(())
}
