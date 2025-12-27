use std::{sync::Arc, time::Duration};

use anyhow::Result;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

use crate::config::DatabaseConfig;

pub type DbPool = Pool<MySql>;
pub type DbPoolManager = Arc<DbPool>;

pub async fn create_pool(config: &DatabaseConfig) -> Result<DbPoolManager> {
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );

    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(Duration::from_secs(30 * 60)) // 30 minutes
        .idle_timeout(Duration::from_secs(10 * 60)) // 10 minutes
        .acquire_timeout(Duration::from_secs(5))
        .test_before_acquire(true)
        .after_connect(|conn, _| {
            Box::pin(async move {
                // we run set statements once per connection, not per query
                sqlx::query(
                    "set sql_mode=(select concat(@@sql_mode, ',PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION')),
                     time_zone='+00:00',
                     names utf8mb4 collate utf8mb4_unicode_ci"
                )
                .execute(conn)
                .await?;

                Ok(())
            })
        })
        .connect(&database_url)
        .await?;

    Ok(Arc::new(pool))
}
