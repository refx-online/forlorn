use crate::infrastructure::redis::RedisConnectionManager;

pub mod announce;

pub async fn publish(
    redis: &RedisConnectionManager,
    channel: &str,
    payload: &str,
) -> redis::RedisResult<()> {
    let mut conn = redis.lock().await;
    redis::cmd("PUBLISH")
        .arg(channel)
        .arg(payload)
        .query_async(&mut *conn)
        .await
}
