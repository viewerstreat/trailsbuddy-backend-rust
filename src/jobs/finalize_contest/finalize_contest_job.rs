use mongodb::{bson::doc, options::FindOptions};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::finish_contest::finish_contest;
use crate::{
    constants::*,
    database::AppDatabase,
    handlers::contest::create::{Contest, ContestStatus},
    utils::get_epoch_ts,
};

pub async fn finalize_contest_job(db: Arc<AppDatabase>) {
    tracing::debug!("initializing finalize contest scheduler job");
    let mut interval = interval(Duration::from_secs(FINALIZE_CONTEST_JOB_INTERVAL));
    loop {
        interval.tick().await;
        check_and_finalize_contest(&db).await;
    }
}

pub async fn check_and_finalize_contest(db: &Arc<AppDatabase>) {
    tracing::debug!("check_and_finalize_contest called");
    let ts = get_epoch_ts() as i64;
    let Ok(status) = ContestStatus::ACTIVE.to_bson() else {
        tracing::debug!("not able to convert ContestStatus to Bson");
        return;
    };
    let filter = doc! {"status": status, "endTime": {"$lte": ts}};
    let options = FindOptions::builder()
        .sort(Some(doc! {"updatedTs": 1}))
        .build();
    let contests = db
        .find::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter), Some(options))
        .await;
    let contests = contests.unwrap_or_else(|e| {
        tracing::debug!("{:?}", e);
        vec![]
    });
    for contest in contests {
        finish_contest(db, &contest).await;
    }
}
