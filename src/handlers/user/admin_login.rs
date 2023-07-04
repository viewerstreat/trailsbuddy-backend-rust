use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use validator::Validate;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JWT_KEYS,
    models::user::AdminUser,
    utils::{get_seq_nxt_val, validation::validate_phonenumber, AppError, ValidatedBody},
};

use super::{check_otp::check_and_update_otp, otp::generate_send_otp};

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
    #[validate(length(equal = "OTP_LENGTH"))]
    otp: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    success: bool,
    data: AdminUser,
    token: String,
}

pub async fn admin_login_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = check_user(&db, &body.phone).await?;
    check_and_update_otp(user.id, &body.otp, &db).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_string()))?;
    let response = LoginResponse {
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

#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
    #[validate(length(min = 1, max = 50))]
    name: String,
}

pub async fn admin_signup_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<SignupRequest>,
) -> Result<Json<JsonValue>, AppError> {
    let id = get_seq_nxt_val(USER_ID_SEQ, &db).await?;
    let user = AdminUser {
        id,
        name: body.name,
        phone: body.phone,
        is_active: true,
    };
    db.insert_one::<AdminUser>(DB_NAME, COLL_ADMIN_USERS, &user, None)
        .await?;
    Ok(Json(json!({"success": true, "message": "Otp generated"})))
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenOtpRequest {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
}

pub async fn admin_generate_otp(
    State(db): State<Arc<AppDatabase>>,
    params: Query<GenOtpRequest>,
) -> Result<Json<JsonValue>, AppError> {
    let user = check_user(&db, &params.phone).await?;
    generate_send_otp(user.id, &db).await?;
    Ok(Json(json!({"success": true, "message": "Otp generated"})))
}

pub async fn get_admin_user_by_id(db: &Arc<AppDatabase>, id: u32) -> anyhow::Result<AdminUser> {
    let filter = doc! {"id": id, "isActive": true};
    let user = db
        .find_one(DB_NAME, COLL_ADMIN_USERS, Some(filter), None)
        .await?
        .ok_or(anyhow::anyhow!("user not found"))?;
    Ok(user)
}
