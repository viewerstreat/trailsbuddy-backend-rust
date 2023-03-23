use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::model::User;
use crate::{
    constants::*,
    utils::{validate_phonenumber, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[cfg_attr(test, double)]
use super::otp::otp_inner;

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyUserReq {
    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,
}

pub async fn verify_user_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<VerifyUserReq>,
) -> Result<Json<JsonValue>, AppError> {
    params.validate()?;
    let filter = Some(doc! {"phone": &params.phone, "isActive": true});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(AppError::NotFound("User not found".into()))?;
    otp_inner::generate_send_otp(user.id, &db).await?;
    Ok(Json(json!({"success": true, "message": "Otp generated"})))
}
