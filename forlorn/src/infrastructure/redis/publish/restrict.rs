use super::publish;
use crate::infrastructure::redis::RedisConnectionManager;

pub async fn restrict(
    redis: &RedisConnectionManager,
    userid: i32,
    reason: &str,
) -> anyhow::Result<()> {
    tracing::warn!("Restricted user id {userid} for {reason}");

    publish(
        redis,
        "refx:restrict",
        &format!("{}|{}", &userid.to_string(), reason),
    )
    .await?;

    Ok(())
}
