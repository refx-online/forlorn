use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn restrict(redis: &RedisConnectionManager, userid: i32, reason: &str) -> anyhow::Result<()> {
    publish(redis, "refx:restrict", &format!("{}|{}", &userid.to_string(), reason)).await?;

    Ok(())
}
