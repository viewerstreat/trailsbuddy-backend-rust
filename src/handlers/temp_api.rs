use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use utoipa::IntoParams;
use validator::Validate;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JWT_KEYS,
    models::{otp::Otp, user::User},
    utils::{get_epoch_ts, validate_phonenumber, AppError},
};

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    user_id: u32,
    name: Option<String>,
}

/// Temporary API to get token
///
/// Returns a JWT token for an user
#[utoipa::path(
    get,
    path = "/api/v1/tempApiGetToken",
    params(Params),
    responses(
        (status = 200, description = "Get JWT token for an user")
    ),
    tag = "Debugging API"
)]
pub async fn temp_api_get_token(params: Query<Params>) -> Result<Json<JsonValue>, AppError> {
    let token = JWT_KEYS.generate_token(params.user_id, params.name.clone())?;
    let res = json!({"success": true, "token": token});
    Ok(Json(res))
}

#[derive(Debug, Deserialize, Validate, IntoParams)]
pub struct OtpParams {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
}

/// Temporary API to get OTP
///
/// Returns the generated OTP for an user
#[utoipa::path(
    get,
    path = "/api/v1/tempApiGetOtp",
    params(OtpParams),
    responses(
        (status = 200, description = "Get OTP for an user")
    ),
    tag = "Debugging API"
)]
pub async fn temp_api_get_otp(
    State(db): State<Arc<AppDatabase>>,
    params: Query<OtpParams>,
) -> Result<Json<JsonValue>, AppError> {
    params
        .validate()
        .map_err(|e| AppError::BadRequestErr(e.to_string()))?;
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
