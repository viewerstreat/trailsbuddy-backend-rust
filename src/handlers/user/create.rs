use axum::{extract::State, http::StatusCode, Json};
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use super::otp::generate_send_otp;
use crate::{
    constants::*,
    models::{user::User, wallet::Money},
    utils::{get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody},
};

use crate::database::AppDatabase;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(custom(function = "validate_phonenumber"))]
    phone: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(email)]
    email: Option<String>,

    #[serde(rename = "profilePic")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    profile_pic: Option<String>,
}

impl CreateUserReq {
    async fn create_user(&self, db: &Arc<AppDatabase>) -> anyhow::Result<User> {
        let id = get_seq_nxt_val(USER_ID_SEQ, db).await?;
        let mut user = User::default();
        user.id = id;
        user.name = self.name.to_owned();
        user.phone = Some(self.phone.to_owned());
        user.email = self.email.clone();
        user.is_active = true;
        user.total_played = Some(0);
        user.contest_won = Some(0);
        user.total_earning = Some(Money::default());
        user.created_ts = Some(get_epoch_ts());
        Ok(user)
    }
}

pub async fn create_user_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateUserReq>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // check if phone already exists in the DB
    check_uniq_phone(&db, body.phone.as_str()).await?;
    // check if email already exists in the DB
    if let Some(email) = &body.email {
        check_uniq_email(&db, email.as_str()).await?;
    }
    let user = body.create_user(&db).await?;
    db.insert_one::<User>(DB_NAME, COLL_USERS, &user, None)
        .await?;
    // generate and send otp to the phone
    generate_send_otp(user.id, &db).await?;
    // return successful response
    let response = (
        StatusCode::CREATED,
        Json(json!({"success": true, "message": "User created"})),
    );
    Ok(response)
}

// check if the given phone already exists in users collection
pub async fn check_uniq_phone(db: &Arc<AppDatabase>, phone: &str) -> Result<(), AppError> {
    let filter = Some(doc! {"phone": phone});
    let result = db
        .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if result.is_some() {
        let err = format!("User already exists with same phone: {}", phone);
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }

    Ok(())
}

// check if the given email already exists in the users collection
pub async fn check_uniq_email(db: &Arc<AppDatabase>, email: &str) -> Result<(), AppError> {
    let filter = Some(doc! {"email": email});
    let result = db
        .find_one::<Document>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if result.is_some() {
        let err = format!("User already exists with same email: {}", email);
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }

    Ok(())
}
