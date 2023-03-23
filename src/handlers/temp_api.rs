use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::user::{model::User, otp::Otp};
use crate::{
    constants::*,
    jwt::JWT_KEYS,
    utils::{get_epoch_ts, validate_phonenumber, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    user_id: u32,
    name: Option<String>,
}

pub async fn temp_api_get_token(params: Query<Params>) -> Result<Json<JsonValue>, AppError> {
    let token = JWT_KEYS.generate_token(params.user_id, params.name.clone())?;
    let res = json!({"success": true, "token": token});
    Ok(Json(res))
}

#[derive(Debug, Deserialize, Validate)]
pub struct OtpParams {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
}

pub async fn temp_api_get_otp(
    State(db): State<Arc<AppDatabase>>,
    params: Query<OtpParams>,
) -> Result<Json<JsonValue>, AppError> {
    params.validate()?;
    let filter = Some(doc! {"phone": &params.phone});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(anyhow::anyhow!("user not found"))?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user.id, "validTill": {"$gte": ts}, "isUsed": false};
    let otp = db
        .find_one::<Otp>(DB_NAME, COLL_OTP, Some(filter), None)
        .await?
        .ok_or(anyhow::anyhow!("Otp not found"))?;
    let res = json!({"success": true, "otp": otp.otp});
    Ok(Json(res))
}
