use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

use super::google_auth_token::GoogleAuthToken;
use crate::constants::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushAndroidNotification {
    color: String,
    image: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushAndroid {
    notification: PushAndroidNotification,
}

#[derive(Debug, Serialize)]
struct PushMessageNotification {
    body: String,
    title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushMessageData {
    screen_name: String,
    screen_params: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushMessage {
    token: String,
    data: PushMessageData,
    notification: PushMessageNotification,
    android: PushAndroid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PushPayload {
    message: PushMessage,
}

impl PushPayload {
    fn new(msg: &str, device: &str) -> Self {
        let push_android_notification = PushAndroidNotification {
            color: PUSH_ICON_COLOR.to_string(),
            image: PUSH_MSG_LOGO_PATH.to_string(),
        };
        let android = PushAndroid {
            notification: push_android_notification,
        };
        let push_message_notification = PushMessageNotification {
            body: msg.to_string(),
            title: PUSH_MESSAGE_TITLE.to_string(),
        };
        let push_message_data = PushMessageData {
            screen_name: "Home".to_string(),
            screen_params: "{\"screen\": \"Notifications\"}".to_string(),
        };
        let message = PushMessage {
            token: device.to_string(),
            data: push_message_data,
            notification: push_message_notification,
            android,
        };
        Self { message }
    }
}

pub async fn send_push_message(
    msg: &str,
    device: &str,
    google_auth_token: &mut GoogleAuthToken,
) -> anyhow::Result<bool> {
    let access_token = google_auth_token.get_access_token().await?;
    let bearer_token = format!("Bearer {}", access_token);
    let payload = PushPayload::new(msg, device);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, bearer_token.as_str().parse()?);
    headers.insert(CONTENT_TYPE, "application/json".parse()?);
    let client = reqwest::Client::new();
    let res = client
        .post(FCM_ENDPOINT)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;
    Ok(res.status().is_success())
}
