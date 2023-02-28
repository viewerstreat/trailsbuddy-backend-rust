use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mockall_double::double;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use super::model::User;
use crate::{
    constants::*,
    utils::{get_epoch_ts, validate_phonenumber, AppError},
};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
pub struct CheckOtpReq {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
    #[validate(length(equal = 6))]
    otp: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: User,
    token: String,
    refresh_token: String,
}

pub async fn check_otp_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<CheckOtpReq>,
) -> Result<(StatusCode, Json<Response>), AppError> {
    params.validate()?;
    let filter = Some(doc! {"phone": params.phone.as_str(), "isActive": true});
    let not_found = format!("User not found with phone: {}", params.phone.as_str());
    let not_found = AppError::BadRequestErr(not_found);
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(not_found)?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user.id, "otp": params.otp.as_str(), "validTill": {"$gte": ts}, "isUsed": false};
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
    let response = Response {
        success: true,
        data: user,
        token: "".to_string(),
        refresh_token: "".to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}
