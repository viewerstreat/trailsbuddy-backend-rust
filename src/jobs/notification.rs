use std::time::Duration;
use tokio::time::interval;

use crate::constants::NOTIFICATION_JOB_INTERVAL;

pub async fn notification_job() {
    tracing::debug!("initializing notification scheduler job");
    let mut interval = interval(Duration::from_secs(NOTIFICATION_JOB_INTERVAL));
    loop {
        interval.tick().await;
        tracing::debug!("Perform notification scheduler job here");
    }
}
