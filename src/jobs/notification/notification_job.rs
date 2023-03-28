use mongodb::{bson::doc, options::FindOptions};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

use super::notification_req::{
    process_new_req, process_ready_req, NotificationReq, NotificationReqStatus,
};
use crate::{
    constants::*, database::AppDatabase, jobs::notification::google_auth_token::GoogleAuthToken,
};

pub async fn notification_job(db: Arc<AppDatabase>) {
    tracing::debug!("initializing notification scheduler job");
    let mut interval = interval(Duration::from_secs(NOTIFICATION_JOB_INTERVAL));
    let mut google_auth_token = GoogleAuthToken::default();
    loop {
        interval.tick().await;
        handle_notification(&db, &mut google_auth_token).await;
    }
}

async fn handle_notification(db: &Arc<AppDatabase>, google_auth_token: &mut GoogleAuthToken) {
    tracing::debug!("running handle_notification scheduler job");
    tokio::join!(
        process_new_batch(&db),
        process_ready_batch(&db, google_auth_token)
    );
}

async fn process_new_batch(db: &Arc<AppDatabase>) {
    let Ok(status) = NotificationReqStatus::NEW.to_bson() else {
        tracing::debug!("not able to convert NotificationReqStatus to bson");
        return;
    };
    let filter = doc! {"status": status};
    let options = FindOptions::builder()
        .sort(Some(doc! {"createdTs": 1}))
        .limit(Some(NOTI_JOB_FETCH_LIMIT))
        .build();
    let requests = db
        .find::<NotificationReq>(
            DB_NAME,
            COLL_NOTIFICATION_REQUESTS,
            Some(filter),
            Some(options),
        )
        .await
        .unwrap_or_else(|e| {
            tracing::debug!("{:?}", e);
            vec![]
        });
    for req in requests {
        process_new_req(db, &req).await;
    }
}

async fn process_ready_batch(db: &Arc<AppDatabase>, google_auth_token: &mut GoogleAuthToken) {
    let Ok(status) = NotificationReqStatus::READY_TO_SEND.to_bson() else {
        tracing::debug!("not able to convert NotificationReqStatus to bson");
        return;
    };
    let filter = doc! {"status": status};
    let options = FindOptions::builder()
        .sort(Some(doc! {"updatedTs": 1}))
        .limit(Some(NOTI_JOB_FETCH_LIMIT))
        .build();
    let requests = db
        .find::<NotificationReq>(
            DB_NAME,
            COLL_NOTIFICATION_REQUESTS,
            Some(filter),
            Some(options),
        )
        .await
        .unwrap_or_else(|e| {
            tracing::debug!("{:?}", e);
            vec![]
        });
    for req in requests {
        process_ready_req(db, &req, google_auth_token).await;
    }
}
