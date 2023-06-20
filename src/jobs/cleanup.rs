use mongodb::bson::doc;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use crate::{constants::*, database::AppDatabase, utils::get_epoch_ts};

/// This function periodically deletes the old OTP & used tokens from database
pub async fn cleanup_job(db: Arc<AppDatabase>) {
    tracing::debug!("initializing cleanup scheduler job");
    // CLEANUP_JOB_INTERVAL is mentioned in seconds
    let mut interval = interval(Duration::from_secs(CLEANUP_JOB_INTERVAL));
    loop {
        interval.tick().await;
        let (otp_result, token_result) = tokio::join!(delete_otp(&db), delete_used_tokens(&db));
        if let Err(err) = otp_result {
            tracing::debug!("Error in otp deletion: {:?}", err);
        }
        if let Err(err) = token_result {
            tracing::debug!("Error in otp deletion: {:?}", err);
        }
    }
}

/// This function deletes the OTP from the database which are older than retention period
async fn delete_otp(db: &Arc<AppDatabase>) -> anyhow::Result<()> {
    let ts = get_epoch_ts();
    // OTP_RETENTION variable is mentioned in number of days
    let cut_off = OTP_RETENTION * 24 * 3600;
    let cut_off = ts - cut_off;
    let filter = doc! {"validTill": {"$lt": cut_off as i64}};
    db.delete_many(DB_NAME, COLL_OTP, filter, None).await?;
    Ok(())
}

/// This function deletes the used tokens which are older than retention period
async fn delete_used_tokens(db: &Arc<AppDatabase>) -> anyhow::Result<()> {
    let ts = get_epoch_ts();
    // USED_TOKEN_RETENTION is mentioned in number of days
    let cut_off = USED_TOKEN_RETENTION * 24 * 3600;
    let cut_off = ts - cut_off;
    let filter = doc! {"validTill": {"$lt": cut_off as i64}};
    db.delete_many(DB_NAME, COLL_USED_TOKENS, filter, None)
        .await?;
    Ok(())
}
