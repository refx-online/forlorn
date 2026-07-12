use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
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
        }
    }
}
