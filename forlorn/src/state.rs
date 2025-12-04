use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::Mutex;

use crate::{
    config::Config,
    infrastructure::{database::DbPoolManager, redis::RedisConnectionManager},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DbPoolManager,
    pub redis: RedisConnectionManager,
    pub score_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: DbPoolManager, redis: RedisConnectionManager) -> Self {
        Self {
            config,
            db,
            redis,
            score_locks: Arc::new(DashMap::new()),
        }
    }
}
