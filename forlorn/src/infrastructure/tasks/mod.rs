use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use tokio::sync::Mutex;

pub async fn cleanup_score_locks(score_locks: Arc<DashMap<String, Arc<Mutex<()>>>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 mins

    loop {
        interval.tick().await;
        // we remove locks that have no waiters
        score_locks.retain(|_, lock| Arc::strong_count(lock) > 1);
    }
}
