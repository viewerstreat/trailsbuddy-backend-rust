use std::time::Duration;
use tokio::time::interval;

use crate::constants::FINALIZE_CONTEST_JOB_INTERVAL;

pub async fn finalize_contest_job() {
    tracing::debug!("initializing finalize contest scheduler job");
    let mut interval = interval(Duration::from_secs(FINALIZE_CONTEST_JOB_INTERVAL));
    loop {
        interval.tick().await;
        tracing::debug!("perform finalize contest scheduler job here");
    }
}
