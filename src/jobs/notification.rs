use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use crate::{constants::NOTIFICATION_JOB_INTERVAL, database::AppDatabase};

pub async fn notification_job(db_client: Arc<AppDatabase>) {
    tracing::debug!("initializing notification scheduler job");
    let mut interval = interval(Duration::from_secs(NOTIFICATION_JOB_INTERVAL));
    loop {
        interval.tick().await;
        tracing::debug!("Perform notification scheduler job here");
    }
}
