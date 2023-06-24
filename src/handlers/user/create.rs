use axum::{extract::State, http::StatusCode, Json};
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use super::otp::generate_send_otp;
use crate::database::AppDatabase;
use crate::{
    constants::*,
    models::{user::User, wallet::Money},
    utils::{
        generate_referral_code, get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError,
        ValidatedBody,
    },
};

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
        let referral_code = create_uniq_referral_code(db, id, &self.name).await?;
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
        user.has_used_referral_code = Some(false);
        user.referral_code = Some(referral_code);
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

/// check if the given phone already exists in users collection
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

/// check if the given email already exists in the users collection
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

/// create an unique referral_code for an user
pub async fn create_uniq_referral_code(
    db: &Arc<AppDatabase>,
    id: u32,
    name: &str,
) -> anyhow::Result<String> {
    let mut loop_counter = 0;
    loop {
        loop_counter += 1;
        let code = generate_referral_code(id, name);
        let filter = Some(doc! {"referralCode": &code});
        let user = db
            .find_one::<Document>(DB_NAME, COLL_USERS, filter.clone(), None)
            .await?;
        let special_referral = db
            .find_one::<Document>(DB_NAME, COLL_SPECIAL_REFERRAL_CODES, filter, None)
            .await?;
        if user.is_none() && special_referral.is_none() {
            return Ok(code);
        }
        if loop_counter >= 3 {
            return Err(anyhow::anyhow!(
                "Not able to generate unique referralCode with 3 retries"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request, routing::post, Router};
    use dotenvy::dotenv;
    use tower::ServiceExt; // for `oneshot` and `ready`

    use super::*;

    #[tokio::test]
    async fn test_create_user_handler_missing_name_field() {
        dotenv().ok();
        // create database client
        let db_client = AppDatabase::new()
            .await
            .expect("Unable to accquire database client");
        let db_client = Arc::new(db_client);
        let app = Router::new()
            .route("/user/create", post(create_user_handler))
            .with_state(db_client);
        let req = Request::builder()
            .uri("/user/create")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{}"#))
            .unwrap();
        let app = app.clone();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_user_handler_missing_phone_field() {
        dotenv().ok();
        // create database client
        let db_client = AppDatabase::new()
            .await
            .expect("Unable to accquire database client");
        let db_client = Arc::new(db_client);
        let app = Router::new()
            .route("/user/create", post(create_user_handler))
            .with_state(db_client);

        let req = Request::builder()
            .uri("/user/create")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name": "TestUser1"}"#))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

// --------------------------------------------------------------------------
// Tests
// - empty object in request body -> 422 Unprocessable Entity
// - request body wihout `name` field -> 422 Unprocessable Entity
// - request body wihout `phone` field -> 422 Unprocessable Entity
// - request body contains name & phone but name does not have any value -> 400
// - request body contains name & phone but phone has alphabetic chars -> 400
// - request body contains name & phone but phone does not have 10 chars -> 400
// - request body has email field & email is not string type - 422
// - request body has email field & email is in invalid format - 400
// - request body has duplicate phone - 400
// - request body has duplicate email - 400
// - request body has profilePic field and in invalid format - 400
// - successful creation following fields to checked
//          - id field has a valid uniq interger from the sequence generator
//          - name field has proper value
//          - phone field has proper value
//          - email field has proper value
//          - profilePic field has proper value
//          - loginScheme = "OTP_BASED"
//          - isActive = true
//          - hasUsedReferralCode = false
//          - referralCode = <Some uniq 8 chars code>
//          - totalPlayed = 0
//          - contestWon = 0
//          - totalEarning = real = 0, bonus = 0
//          - otps collection have a new otp for the user
// --------------------------------------------------------------------------
