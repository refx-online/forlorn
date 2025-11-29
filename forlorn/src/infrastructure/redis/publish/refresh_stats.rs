use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn refresh_stats(redis: &RedisConnectionManager, userid: i32) -> anyhow::Result<()> {
    publish(redis, "refx:refresh_stats", &userid.to_string()).await?;

    Ok(())
}
