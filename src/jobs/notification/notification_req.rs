use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::{google_auth_token::GoogleAuthToken, push_message::send_push_message};
use crate::{
    constants::*,
    database::AppDatabase,
    handlers::notification::get_noti::Notifications,
    models::user::User,
    utils::{deserialize_helper, get_epoch_ts, parse_object_id, replace_placeholders},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum NotificationType {
    PUSH_MESSAGE,
    SMS_MESSAGE,
    EMAIL_MESSAGE,
}

impl NotificationType {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum NotificationReqStatus {
    NEW,
    READY_TO_SEND,
    SENT,
    ERROR,
}

impl NotificationReqStatus {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationReq {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_helper")]
    #[serde(default)]
    pub _id: Option<String>,
    pub event_name: String,
    pub notification_type: NotificationType,
    pub user_id: u32,
    pub data: HashMap<String, String>,
    pub status: NotificationReqStatus,
    pub final_message: Option<String>,
    pub fcm_tokens: Option<Vec<String>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub created_ts: Option<u64>,
    pub updated_ts: Option<u64>,
}

impl NotificationReq {
    pub fn new(user_id: u32, event_name: &str, data: HashMap<String, String>) -> Self {
        let ts = get_epoch_ts();
        Self {
            _id: None,
            event_name: event_name.to_string(),
            notification_type: NotificationType::PUSH_MESSAGE,
            user_id,
            data,
            status: NotificationReqStatus::NEW,
            final_message: None,
            fcm_tokens: None,
            error_message: None,
            retry_count: 0,
            created_ts: Some(ts),
            updated_ts: None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationContent {
    event_name: String,
    content: String,
}

pub async fn process_new_req(db: &Arc<AppDatabase>, req: &NotificationReq) {
    tracing::debug!("process_new_req: {:?}", req);
    if req.notification_type != NotificationType::PUSH_MESSAGE {
        return;
    }
    let filter = doc! {"eventName": &req.event_name};
    let content = db
        .find_one::<NotificationContent>(DB_NAME, COLL_NOTIFICATION_CONTENTS, Some(filter), None)
        .await;
    let content = match content {
        Ok(content) => content,
        Err(e) => {
            tracing::debug!("{:?}", e);
            return update_error(db, req, "not able to get content").await;
        }
    };
    let content = match content {
        Some(content) => content,
        None => {
            return update_error(db, req, "content not found").await;
        }
    };
    let filter = doc! {"id": req.user_id, "isActive": true};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await;
    let user = match user {
        Ok(user) => user,
        Err(e) => {
            tracing::debug!("{:?}", e);
            return update_error(db, req, "not able to get user").await;
        }
    };
    let user = match user {
        Some(user) => user,
        None => {
            return update_error(db, req, "user not found").await;
        }
    };
    let fcm_tokens = user.fcm_tokens.unwrap_or_default();
    if fcm_tokens.is_empty() {
        return update_error(db, req, "fcm_tokens not found").await;
    }
    let data = req.data.clone();
    let final_message = replace_placeholders(&content.content, data).unwrap_or_else(|e| {
        tracing::debug!("{:?}", e);
        String::new()
    });
    if final_message.is_empty() {
        return update_error(db, req, "final_message not found").await;
    }
    update_ready(db, req, &fcm_tokens, &final_message).await;
}

pub async fn process_ready_req(
    db: &Arc<AppDatabase>,
    req: &NotificationReq,
    google_auth_token: &mut GoogleAuthToken,
) {
    tracing::debug!("process_ready_req: {:?}", req);
    if req.notification_type != NotificationType::PUSH_MESSAGE {
        return;
    }
    let Some(final_message) = req.final_message.as_ref() else {
        return update_error(db, req, "final_message not found").await;
    };
    if final_message.is_empty() {
        return update_error(db, req, "final_message not found").await;
    }
    let Some(fcm_tokens) = req.fcm_tokens.as_ref() else {
        return update_error(db, req, "fcmTokens not found").await;
    };
    if fcm_tokens.is_empty() {
        return update_error(db, req, "fcmTokens not found").await;
    }
    let mut results = Vec::with_capacity(fcm_tokens.len());
    for device in fcm_tokens {
        let r = send_push_message(final_message, device, google_auth_token).await;
        results.push(r);
    }
    if results
        .iter()
        .all(|r| if let Ok(v) = r { !v } else { true })
    {
        return update_error(db, req, "not able to send_push_message").await;
    }
    update_completed(db, req, final_message).await;
}

pub async fn update_ready(
    db: &Arc<AppDatabase>,
    req: &NotificationReq,
    fcm_tokens: &Vec<String>,
    final_message: &str,
) {
    let Some(id) = req._id.as_ref() else {
        tracing::debug!("_id not present in req");
        return;
    };
    let Ok(oid) = parse_object_id(id, "not able to parse") else {
        tracing::debug!("not able to parse _id");
        return;
    };
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": oid};
    let Ok(status) = NotificationReqStatus::READY_TO_SEND.to_bson() else {
        tracing::debug!("not able to convert NotificationReqStatus to bson");
        return;
    };
    let update = doc! {
        "$set": {
            "status": status,
            "fcmTokens": fcm_tokens,
            "finalMessage": final_message,
            "updatedTs": ts
        }
    };
    if let Err(e) = db
        .update_one(DB_NAME, COLL_NOTIFICATION_REQUESTS, filter, update, None)
        .await
    {
        tracing::debug!("{:?}", e);
        return update_error(db, req, "not able to update request ready").await;
    }
}

pub async fn update_completed(db: &Arc<AppDatabase>, req: &NotificationReq, msg: &str) {
    let Some(id) = req._id.as_ref() else {
        return update_error(db, req, "_id not present in req").await;
    };
    let Ok(oid) = parse_object_id(id, "not able to parse") else {
        return update_error(db, req, "not able to parse _id").await;
    };
    let notification = Notifications::new_push(req.user_id, &req.event_name, msg);
    if let Err(e) = db
        .insert_one::<Notifications>(DB_NAME, COLL_NOTIFICATIONS, &notification, None)
        .await
    {
        tracing::debug!("{:?}", e);
        return update_error(db, req, "not able to insert into notifications").await;
    }
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": oid};
    let Ok(status) = NotificationReqStatus::SENT.to_bson() else {
        return update_error(db, req, "not able to convert NotificationReqStatus to bson").await;
    };
    let update = doc! {
        "$set": {
            "status": status,
            "updatedTs": ts
        }
    };
    if let Err(e) = db
        .update_one(DB_NAME, COLL_NOTIFICATION_REQUESTS, filter, update, None)
        .await
    {
        tracing::debug!("{:?}", e);
        return update_error(db, req, "not able to update request SENT").await;
    }
}

pub async fn update_error(db: &Arc<AppDatabase>, req: &NotificationReq, error_message: &str) {
    let Some(id) = req._id.as_ref() else {
        tracing::debug!("_id not present in req");
        return;
    };
    let Ok(oid) = parse_object_id(id, "") else {
        tracing::debug!("not able to parse _id");
        return;
    };
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": oid};
    let retry_count = req.retry_count + 1;
    let mut update = doc! {
        "errorMessage": error_message,
        "retryCount": retry_count,
        "updatedTs": ts
    };
    if retry_count >= NOTI_JOB_MAX_RETRY_COUNT {
        let Ok(status) = NotificationReqStatus::ERROR.to_bson() else {
            tracing::debug!("not able to convert NotificationReqStatus to bson");
            return;
        };
        update.insert("status", status);
    }
    update = doc! {"$set": update};
    let r = db
        .update_one(DB_NAME, COLL_NOTIFICATION_REQUESTS, filter, update, None)
        .await;
    if let Err(e) = r {
        tracing::debug!("{:?}", e);
    }
}
