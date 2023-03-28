use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    jobs::notification::notification_req::NotificationType,
    jwt::JwtClaims,
    utils::{deserialize_helper, get_epoch_ts, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_helper")]
    #[serde(default)]
    _id: Option<String>,
    event_name: String,
    notification_type: NotificationType,
    user_id: u32,
    message: String,
    is_read: bool,
    is_cleared: bool,
    created_ts: Option<u64>,
    updated_ts: Option<u64>,
}

impl Notifications {
    pub fn new_push(user_id: u32, event_name: &str, msg: &str) -> Self {
        let ts = get_epoch_ts();
        Self {
            _id: None,
            event_name: event_name.to_string(),
            notification_type: NotificationType::PUSH_MESSAGE,
            user_id,
            message: msg.to_string(),
            is_read: false,
            is_cleared: false,
            created_ts: Some(ts),
            updated_ts: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNotiReq {
    page_index: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct GetNotiResp {
    success: bool,
    data: Vec<Notifications>,
}

pub async fn get_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<GetNotiReq>,
) -> Result<Json<GetNotiResp>, AppError> {
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let skip = page_index * page_size;
    let sort = doc! {"_id": -1};
    let mut options = FindOptions::default();
    options.sort = Some(sort);
    options.skip = Some(skip);
    options.limit = Some(page_size as i64);
    let options = Some(options);
    let push_message = NotificationType::PUSH_MESSAGE.to_bson()?;
    let filter = doc! {"userId": claims.id, "isCleared": false, "notificationType": push_message};
    let result = db
        .find::<Notifications>(DB_NAME, COLL_NOTIFICATIONS, Some(filter), options)
        .await?;
    let res = GetNotiResp {
        success: true,
        data: result,
    };
    Ok(Json(res))
}
