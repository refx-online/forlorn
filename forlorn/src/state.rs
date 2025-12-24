use std::sync::Arc;

use dashmap::{DashMap, DashSet};
use storage::Storage;
use tokio::sync::Mutex;

use crate::{
    config::Config,
    infrastructure::{
        database::DbPoolManager,
        redis::{RedisConnectionManager, RedisPubsubManager},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub storage: Storage,
    pub db: DbPoolManager,
    pub redis: RedisConnectionManager,
    pub subscriber: RedisPubsubManager,
    pub score_locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
    pub unsubmitted_maps: Arc<DashSet<String>>,
    pub needs_update_maps: Arc<DashSet<String>>,
}

impl AppState {
    pub fn new(
        config: Arc<Config>,
        storage: Storage,
        db: DbPoolManager,
        redis: RedisConnectionManager,
        subscriber: RedisPubsubManager,
    ) -> Self {
        Self {
            config,
            storage,
            db,
            redis,
            subscriber,
            score_locks: Arc::new(DashMap::new()),
            unsubmitted_maps: Arc::new(DashSet::new()),
            needs_update_maps: Arc::new(DashSet::new()),
        }
    }
}
