use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
    jwt::JWT_KEYS,
    models::user::{LoginScheme, User},
    utils::{get_epoch_ts, validate_phonenumber, AppError},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
pub struct CheckOtpReq {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
    #[validate(length(equal = 6))]
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
    update_login_time(&db, user.id, LoginScheme::OTP_BASED).await?;

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

// update lastLoginTime for user
pub async fn update_login_time(
    db: &Arc<AppDatabase>,
    id: u32,
    login_scheme: LoginScheme,
) -> anyhow::Result<()> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"id": id};
    let update = doc! {"$set": {"lastLoginTime": ts, "loginScheme": login_scheme.to_string()}};
    db.update_one(DB_NAME, COLL_USERS, filter, update, None)
        .await?;
    Ok(())
}
