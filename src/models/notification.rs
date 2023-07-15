use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::utils::{deserialize_helper, get_epoch_ts};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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
pub struct NotificationContent {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
