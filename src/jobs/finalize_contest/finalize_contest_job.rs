use mongodb::{bson::doc, options::FindOptions};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::finish_contest::finish_contest;
use crate::{
    constants::*,
    database::AppDatabase,
    models::contest::{Contest, ContestStatus},
    utils::get_epoch_ts,
};

pub async fn finalize_contest_job(db: Arc<AppDatabase>) {
    tracing::debug!("initializing finalize contest scheduler job");
    let mut interval = interval(Duration::from_secs(FINALIZE_CONTEST_JOB_INTERVAL));
    loop {
        interval.tick().await;
        let result = check_and_finalize_contest(&db).await;
        if result.is_err() {
            tracing::debug!("Error in finalize_contest_job => {:?}", result.err());
        }
    }
}

pub async fn check_and_finalize_contest(db: &Arc<AppDatabase>) -> anyhow::Result<()> {
    tracing::debug!("check_and_finalize_contest called");
    let ts = get_epoch_ts() as i64;
    let status = ContestStatus::ACTIVE.to_bson()?;
    let filter = doc! {"status": status, "endTime": {"$lte": ts}};
    let options = FindOptions::builder()
        .sort(Some(doc! {"updatedTs": 1}))
        .build();
    let contests = db
        .find::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter), Some(options))
        .await
        .unwrap_or_default();
    for contest in contests {
        finish_contest(db, &contest).await;
    }
    Ok(())
}
