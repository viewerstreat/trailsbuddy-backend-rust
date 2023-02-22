use axum::{extract::State, http::StatusCode, Json};
use mongodb::{
    bson::{doc, Document},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    constants::*,
    utils::{get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginScheme {
    OtpBased,
    Google,
    Facebook,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSchema {
    id: u32,
    name: String,
    phone: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,

    #[serde(rename = "profilePic")]
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_pic: Option<String>,

    #[serde(rename = "loginScheme")]
    login_scheme: LoginScheme,

    #[serde(rename = "isActive")]
    is_active: bool,

    // last_login_time: Option<u64>,
    // has_used_referral_code: Option<bool>,
    // referral_code: Option<String>,
    // referred_by: Option<String>,
    #[serde(rename = "totalPlayed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    total_played: Option<u32>,

    #[serde(rename = "contestWon")]
    #[serde(skip_serializing_if = "Option::is_none")]
    contest_won: Option<u32>,

    #[serde(rename = "totalEarning")]
    #[serde(skip_serializing_if = "Option::is_none")]
    total_earning: Option<u32>,

    // fcm_tokens: Option<Vec<String>>,
    #[serde(rename = "createdTs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    created_ts: Option<u64>,

    #[serde(rename = "updatedTs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_ts: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserReqBody {
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

pub async fn create_user_handler(
    State(client): State<Client>,
    ValidatedBody(body): ValidatedBody<CreateUserReqBody>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // check if phone already exists in the DB
    check_uniq_phone(&client, body.phone.as_str()).await?;
    // check if email already exists in the DB
    if body.email.is_some() {
        let email = body.email.unwrap();
        check_uniq_email(&client, &email).await?;
    }

    // let user_coll = &client
    //     .database(DB_NAME)
    //     .collection::<UserSchema>(COLL_USERS);
    // let user = UserSchema::default();
    // user_coll.insert_one(user, None).await?;

    // let id = get_seq_nxt_val(USER_ID_SEQ, &client).await?;
    // println!("User id is : {id}");
    Ok((
        StatusCode::CREATED,
        Json(json!({"success": true, "message": "User created"})),
    ))
}

// check if the given phone already exists in users collection
async fn check_uniq_phone(client: &Client, phone: &str) -> Result<(), AppError> {
    let user_coll = &client.database(DB_NAME).collection::<Document>(COLL_USERS);
    let check_ph_result = user_coll.find_one(doc! {"phone": phone}, None).await?;
    if check_ph_result.is_some() {
        return Err(AppError::BadRequestErr(
            "User already exists with same phone: {phone}",
        ));
    }

    Ok(())
}

// check if the given email already exists in the users collection
async fn check_uniq_email(client: &Client, email: &str) -> Result<(), AppError> {
    let user_coll = &client.database(DB_NAME).collection::<Document>(COLL_USERS);
    let result = user_coll.find_one(doc! {"email": email}, None).await?;
    if result.is_some() {
        return Err(AppError::BadRequestErr(
            "User already exists with same email: {email}",
        ));
    }

    Ok(())
}
