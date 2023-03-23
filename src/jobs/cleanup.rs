use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use crate::{constants::CLEANUP_JOB_INTERVAL, database::AppDatabase};

pub async fn cleanup_job(db_client: Arc<AppDatabase>) {
    tracing::debug!("initializing cleanup scheduler job");
    let mut interval = interval(Duration::from_secs(CLEANUP_JOB_INTERVAL));
    loop {
        interval.tick().await;
        tracing::debug!("perform cleanup scheduler job here");
    }
}
