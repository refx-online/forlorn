use std::sync::Arc;

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
        .connect(&database_url)
        .await?;

    Ok(Arc::new(pool))
}
