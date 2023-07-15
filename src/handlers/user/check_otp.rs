use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::user::login::update_user_login,
    jwt::JWT_KEYS,
    models::*,
    utils::{get_epoch_ts, AppError},
};

/// Check user otp
///
/// Check if the provided otp is valid for the given phone
/// and the valid user exists for the phone
/// If yes then generate token and refreshToken and return success response
#[utoipa::path(
    get,
    path = "/api/v1/user/checkOtp",
    params(CheckOtpReq),
    responses(
        (status = StatusCode::OK, description = "Login successful", body = LoginResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn check_otp_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<CheckOtpReq>,
) -> Result<Json<LoginResponse>, AppError> {
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

    let response = LoginResponse {
        success: true,
        data: user,
        token,
        refresh_token: Some(refresh_token),
    };

    Ok(Json(response))
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
