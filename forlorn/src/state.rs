use std::sync::Arc;

use dashmap::DashSet;
use dogstatsd::Client as DatadogClient;
use rslock::LockManager;
use storage::Storage;

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
    pub score_locks: LockManager,
    pub metrics: Arc<DatadogClient>,
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
        score_locks: LockManager,
        metrics: Arc<DatadogClient>,
    ) -> Self {
        Self {
            config,
            storage,
            db,
            redis,
            subscriber,
            score_locks,
            metrics,
            unsubmitted_maps: Arc::new(DashSet::new()),
            needs_update_maps: Arc::new(DashSet::new()),
        }
    }
}
