use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::user::{check_otp::check_and_update_otp, otp::generate_send_otp_admin},
    jwt::JWT_KEYS,
    models::*,
    utils::{get_seq_nxt_val, AppError, ValidatedBody},
};

/// Admin login
///
/// Login for admin user with phone and otp
#[utoipa::path(
    post,
    path = "/api/v1/admin/login",
    request_body = CheckOtpReq,
    responses(
        (status = StatusCode::OK, description = "Use successfully redeems referral code", body = AdminLoginResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::NOT_FOUND, description = "User/otp not found", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn admin_login_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CheckOtpReq>,
) -> Result<Json<AdminLoginResponse>, AppError> {
    let user = check_user(&db, &body.phone).await?;
    check_and_update_otp(user.id, &body.otp, &db).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_string()))?;
    let response = AdminLoginResponse {
        success: true,
        data: user,
        token,
    };
    Ok(Json(response))
}

async fn check_user(db: &Arc<AppDatabase>, phone: &str) -> Result<AdminUser, AppError> {
    let filter = doc! {"phone": &phone, "isActive": true};
    let user = db
        .find_one::<AdminUser>(DB_NAME, COLL_ADMIN_USERS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("User not found".into()))?;
    Ok(user)
}

/// Admin signup
///
/// Creates a new admin user
#[utoipa::path(
    post,
    path = "/api/v1/admin/signup",
    request_body = AdminSignupRequest,
    responses(
        (status = StatusCode::OK, description = "Use successfully redeems referral code", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn admin_signup_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<AdminSignupRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let id = get_seq_nxt_val(USER_ID_SEQ, &db).await?;
    let filter = doc! {"phone": &body.phone};
    let user = db
        .find_one::<AdminUser>(DB_NAME, COLL_ADMIN_USERS, Some(filter), None)
        .await?;
    if user.is_some() {
        let err = format!("User already exists with same phone: {}", body.phone);
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }
    let user = AdminUser {
        id,
        name: body.name,
        phone: body.phone,
        is_active: true,
    };
    db.insert_one::<AdminUser>(DB_NAME, COLL_ADMIN_USERS, &user, None)
        .await?;
    generate_send_otp_admin(user.id, &db).await?;
    let res = GenericResponse {
        success: true,
        message: "Otp generated".to_owned(),
    };
    Ok(Json(res))
}

/// Verify phone and generate otp
///
/// Verify phone if it is valid admin user's phone and generate an otp
#[utoipa::path(
    get,
    path = "/api/v1/admin/generateOtp",
    params(VerifyUserReq),
    responses(
        (status = StatusCode::OK, description = "Valid user & OTP is generated", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn admin_generate_otp(
    State(db): State<Arc<AppDatabase>>,
    params: Query<VerifyUserReq>,
) -> Result<Json<GenericResponse>, AppError> {
    let user = check_user(&db, &params.phone).await?;
    generate_send_otp_admin(user.id, &db).await?;
    let res = GenericResponse {
        success: true,
        message: "Otp generated".to_owned(),
    };
    Ok(Json(res))
}
