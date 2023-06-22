use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::otp::generate_send_otp;
use crate::{
    constants::*,
    database::AppDatabase,
    models::user::User,
    utils::{validate_phonenumber, AppError},
};

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyUserReq {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
}

/// Verify if there is any active user with the provided phone
/// If present then generate and send an otp to the phone number
/// and return success response
pub async fn verify_user_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<VerifyUserReq>,
) -> Result<Json<JsonValue>, AppError> {
    params
        .validate()
        .map_err(|err| AppError::BadRequestErr(err.to_string()))?;
    let filter = Some(doc! {"phone": &params.phone, "isActive": true});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(AppError::NotFound("User not found".into()))?;
    generate_send_otp(user.id, &db).await?;
    Ok(Json(json!({"success": true, "message": "Otp generated"})))
}
