use std::sync::Arc;

use crate::{
    config::Config,
    infrastructure::{database::DbPoolManager, redis::RedisConnectionManager},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DbPoolManager,
    pub redis: RedisConnectionManager,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: DbPoolManager, redis: RedisConnectionManager) -> Self {
        Self { config, db, redis }
    }
}
