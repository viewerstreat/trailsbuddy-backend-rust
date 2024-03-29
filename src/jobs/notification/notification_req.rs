use mongodb::bson::doc;
use std::sync::Arc;

use super::{google_auth_token::GoogleAuthToken, push_message::send_push_message};
use crate::{
    constants::*,
    database::AppDatabase,
    models::{
        notification::{
            NotificationContent, NotificationReq, NotificationReqStatus, NotificationType,
            Notifications,
        },
        user::User,
    },
    utils::{get_epoch_ts, parse_object_id, replace_placeholders},
};

/// This function processes a single notification request which is in `NEW` status
pub async fn process_new_req(db: &Arc<AppDatabase>, req: &NotificationReq) {
    tracing::debug!("process_new_req: {:?}", req);
    // for now handle only PUSH_MESSAGE otherwise exit
    if req.notification_type != NotificationType::PUSH_MESSAGE {
        return;
    }
    let filter = doc! {"eventName": &req.event_name};
    // get the content for the particular event_name from notificationContent collection
    let content = db
        .find_one::<NotificationContent>(DB_NAME, COLL_NOTIFICATION_CONTENTS, Some(filter), None)
        .await;
    // if content is not found for the event_name then raise error
    let content = match content {
        Ok(content) => match content {
            Some(content) => content,
            None => {
                return update_error(db, req, "content not found").await;
            }
        },
        Err(e) => {
            tracing::debug!("{:?}", e);
            return update_error(db, req, "not able to get content").await;
        }
    };
    let filter = doc! {"id": req.user_id, "isActive": true};
    // get the user from users collection
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await;
    // if user is not found then raise error
    let user = match user {
        Ok(user) => match user {
            Some(user) => user,
            None => {
                return update_error(db, req, "user not found").await;
            }
        },
        Err(e) => {
            tracing::debug!("{:?}", e);
            return update_error(db, req, "not able to get user").await;
        }
    };
    let fcm_tokens = user.fcm_tokens.unwrap_or_default();
    // if user details doesn't contain fcm tokens then raise error
    if fcm_tokens.is_empty() {
        return update_error(db, req, "fcm_tokens not found").await;
    }
    let data = req.data.clone();
    // prepare the final message to be sent out
    let final_message = replace_placeholders(&content.content, data).unwrap_or_else(|e| {
        tracing::debug!("{:?}", e);
        String::new()
    });
    // if the final_message is empty then raise error
    if final_message.is_empty() {
        return update_error(db, req, "final_message not found").await;
    }
    // update the request to READY_TO_SEND status
    update_ready(db, req, &fcm_tokens, &final_message).await;
}

/// This function processes single notification request which is in READY_TO_SEND status
pub async fn process_ready_req(
    db: &Arc<AppDatabase>,
    req: &NotificationReq,
    google_auth_token: &mut GoogleAuthToken,
) {
    tracing::debug!("process_ready_req: {:?}", req);
    // for now handle only PUSH_MESSAGE otherwise exit
    if req.notification_type != NotificationType::PUSH_MESSAGE {
        return;
    }
    // if final_message is not present then raise error
    let Some(final_message) = req.final_message.as_ref() else {
        return update_error(db, req, "final_message not found").await;
    };
    // if final_message is empty then raise error
    if final_message.is_empty() {
        return update_error(db, req, "final_message not found").await;
    }
    // if fcm_tokens are not found then raise error
    let Some(fcm_tokens) = req.fcm_tokens.as_ref() else {
        return update_error(db, req, "fcmTokens not found").await;
    };
    // if fcm_tokens are not found then raise error
    if fcm_tokens.is_empty() {
        return update_error(db, req, "fcmTokens not found").await;
    }
    let mut results = Vec::with_capacity(fcm_tokens.len());
    // for each fcm_tokens send out the push message
    for device in fcm_tokens {
        let r = send_push_message(final_message, device, google_auth_token).await;
        results.push(r);
    }
    // if sending push message failed for all fcm tokens then raise error
    if results
        .iter()
        .all(|r| if let Ok(v) = r { !v } else { true })
    {
        return update_error(db, req, "not able to send_push_message").await;
    }
    // update the request to completed status
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

/// This function updates the notification request with error messages and increase the retry_count
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
    // When the retry_count exceeds the max limit update the status to ERROR as well
    // so that it will not be retried any further
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
