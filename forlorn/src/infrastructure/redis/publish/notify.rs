use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

/// sent packet 24 not 25
pub async fn notify(
    redis: &RedisConnectionManager,
    userid: i32,
    message: &str,
) -> anyhow::Result<()> {
    publish(
        redis,
        "refx:notify",
        &format!("{}|{}", &userid.to_string(), message),
    )
    .await?;

    Ok(())
}
