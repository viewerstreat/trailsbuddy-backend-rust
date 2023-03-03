use axum::{
    extract::{Query, State},
    Json,
};
use mockall_double::double;
use mongodb::bson::serde_helpers::hex_string_as_object_id;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};

use crate::{constants::*, jwt::JwtClaims, utils::AppError};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum NotificationType {
    PUSH_MESSAGE,
    SMS_MESSAGE,
    EMAIL_MESSAGE,
}

impl Display for NotificationType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::PUSH_MESSAGE => write!(f, "PUSH_MESSAGE"),
            Self::SMS_MESSAGE => write!(f, "SMS_MESSAGE"),
            Self::EMAIL_MESSAGE => write!(f, "EMAIL_MESSAGE"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notifications {
    #[serde(rename = "_id")]
    #[serde(deserialize_with = "hex_string_as_object_id::deserialize")]
    id: String,
    event_name: String,
    notification_type: NotificationType,
    user_id: u32,
    message: String,
    is_read: bool,
    is_cleared: bool,
    created_ts: u64,
    updated_ts: u64,
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
    let push_message = NotificationType::PUSH_MESSAGE.to_string();
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
