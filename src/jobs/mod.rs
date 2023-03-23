use std::sync::Arc;

use self::{
    cleanup::cleanup_job, finalize_contest::finalize_contest_job, notification::notification_job,
};
use crate::database::AppDatabase;

pub mod cleanup;
pub mod finalize_contest;
pub mod notification;

pub fn spawn_all_jobs(db_client: Arc<AppDatabase>) {
    {
        let db_client = db_client.clone();
        tokio::spawn(async {
            cleanup_job(db_client).await;
        });
    }

    {
        let db_client = db_client.clone();
        tokio::spawn(async {
            notification_job(db_client).await;
        });
    }

    tokio::spawn(async {
        finalize_contest_job(db_client).await;
    });
}
