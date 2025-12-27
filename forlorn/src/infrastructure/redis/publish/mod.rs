use crate::infrastructure::redis::RedisConnectionManager;

pub mod announce;
pub mod notify;
pub mod refresh_stats;
pub mod restrict;
pub mod score;

pub async fn publish(
    redis: &RedisConnectionManager,
    channel: &str,
    payload: &str,
) -> redis::RedisResult<()> {
    let mut conn = redis.lock().await;
    match redis::cmd("PUBLISH")
        .arg(channel)
        .arg(payload)
        .query_async(&mut *conn)
        .await
    {
        Ok(result) => Ok(result),
        Err(_) => {
            drop(conn);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            let mut conn = redis.lock().await;
            redis::cmd("PUBLISH")
                .arg(channel)
                .arg(payload)
                .query_async(&mut *conn)
                .await
        },
    }
}
