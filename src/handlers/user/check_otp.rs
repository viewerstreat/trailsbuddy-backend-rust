use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::database::AppDatabase;
use crate::{
    constants::*,
    jwt::JWT_KEYS,
    models::user::{LoginScheme, User},
    utils::{get_epoch_ts, validate_phonenumber, AppError},
};

use super::login::update_user_login;

#[derive(Debug, Deserialize, Validate)]
pub struct CheckOtpReq {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
    #[validate(length(equal = "OTP_LENGTH"))]
    otp: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Response {
    success: bool,
    data: User,
    token: String,
    refresh_token: String,
}

/// Check if the provided otp is valid for the given phone
/// and the valid user exists for the phone
/// If yes then generate token and refreshToken and return success response
pub async fn check_otp_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<CheckOtpReq>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    params
        .validate()
        .map_err(|err| AppError::BadRequestErr(err.to_string()))?;
    let filter = Some(doc! {"phone": params.phone.as_str(), "isActive": true});
    let not_found = format!("User not found with phone: {}", params.phone.as_str());
    let not_found = AppError::NotFound(not_found);
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(not_found)?;
    check_and_update_otp(user.id, params.otp.as_str(), &db).await?;
    update_user_login(&db, user.id, LoginScheme::OTP_BASED).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_owned()))?;
    let refresh_token = JWT_KEYS.generate_refresh_token(user.id, None)?;

    let response = Response {
        success: true,
        data: user,
        token,
        refresh_token,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn check_and_update_otp(
    user_id: u32,
    otp: &str,
    db: &Arc<AppDatabase>,
) -> Result<(), AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user_id, "otp": &otp, "validTill": {"$gte": ts}, "isUsed": false};
    let update = doc! {"$set": {"isUsed": true, "updateTs": ts}};
    let result = db
        .update_one(DB_NAME, COLL_OTP, filter, update, None)
        .await?;
    if result.matched_count == 0 {
        let err = format!("Not valid otp");
        return Err(AppError::NotFound(err));
    }
    if result.modified_count == 0 {
        let err = anyhow::anyhow!("Not able to update otp in DB");
        return Err(AppError::AnyError(err));
    }
    Ok(())
}
