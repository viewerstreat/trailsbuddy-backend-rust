// use std::sync::Arc;

// use anyhow::anyhow;
// use axum::{extract::State, http::StatusCode, Json};
// use mongodb::{
//     bson::{doc, Document},
//     Client,
// };
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
// use validator::Validate;

// use crate::{
//     constants::*,
//     utils::{
//         generate_otp, get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody,
//     },
// };

// #[derive(Debug, Default, Serialize, Deserialize)]
// #[allow(non_camel_case_types)]
// pub enum LoginScheme {
//     #[default]
//     OTP_BASED,
//     GOOGLE,
//     FACEBOOK,
// }

// #[derive(Debug, Default, Serialize, Deserialize)]
// pub struct UserSchema {
//     id: u32,
//     name: String,
//     phone: String,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     email: Option<String>,

//     #[serde(rename = "profilePic")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     profile_pic: Option<String>,

//     #[serde(rename = "loginScheme")]
//     login_scheme: LoginScheme,

//     #[serde(rename = "isActive")]
//     is_active: bool,

//     #[serde(rename = "lastLoginTime")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     last_login_time: Option<u64>,

//     #[serde(rename = "hasUsedReferralCode")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     has_used_referral_code: Option<bool>,

//     #[serde(rename = "referralCode")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     referral_code: Option<String>,

//     #[serde(rename = "referredBy")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     referred_by: Option<String>,

//     #[serde(rename = "totalPlayed")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     total_played: Option<u32>,

//     #[serde(rename = "contestWon")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     contest_won: Option<u32>,

//     #[serde(rename = "totalEarning")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     total_earning: Option<u32>,

//     // fcm_tokens: Option<Vec<String>>,
//     #[serde(rename = "createdTs")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     created_ts: Option<u64>,

//     #[serde(rename = "updatedTs")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     updated_ts: Option<u64>,
// }

// impl UserSchema {
//     async fn from_create_user_req_body(
//         body: &CreateUserReqBody,
//         client: DbInterface,
//     ) -> anyhow::Result<Self> {
//         let id = get_seq_nxt_val(USER_ID_SEQ).await?;
//         let mut user = Self::default();
//         user.id = id;
//         user.name = body.name.to_owned();
//         user.phone = body.phone.to_owned();
//         user.is_active = true;
//         user.total_played = Some(0);
//         user.contest_won = Some(0);
//         user.total_earning = Some(0);
//         user.created_ts = Some(get_epoch_ts());
//         Ok(user)
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Validate)]
// pub struct CreateUserReqBody {
//     #[validate(length(min = 1, max = 50))]
//     name: String,

//     #[validate(custom(function = "validate_phonenumber"))]
//     phone: String,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[validate(email)]
//     email: Option<String>,

//     #[serde(rename = "profilePic")]
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[validate(url)]
//     profile_pic: Option<String>,
// }

// pub async fn create_user_handler(
//     State(client): State<DbInterface>,
//     ValidatedBody(body): ValidatedBody<CreateUserReqBody>,
// ) -> Result<(StatusCode, Json<Value>), AppError> {
//     // check if phone already exists in the DB
//     check_uniq_phone(client.clone(), body.phone.as_str()).await?;
//     // check if email already exists in the DB
//     if let Some(email) = &body.email {
//         check_uniq_email(client.clone(), email.as_str()).await?;
//     }
//     // create typed collection for UserSchema
//     let user_coll = client
//         .database(DB_NAME)
//         .collection::<UserSchema>(COLL_USERS);
//     // get the user from body
//     let user = UserSchema::from_create_user_req_body(&body, client.clone()).await?;
//     // insert into database
//     user_coll.insert_one(&user, None).await?;
//     // generate and send otp to the phone
//     generate_send_otp(user.id, client.clone()).await?;
//     // return successful response
//     Ok((
//         StatusCode::CREATED,
//         Json(json!({"success": true, "message": "User created"})),
//     ))
// }

// // check if the given phone already exists in users collection
// async fn check_uniq_phone(client: DbInterface, phone: &str) -> Result<(), AppError> {
//     let user_coll = &client.database(DB_NAME).collection::<Document>(COLL_USERS);
//     let check_ph_result = user_coll.find_one(doc! {"phone": phone}, None).await?;
//     if check_ph_result.is_some() {
//         return Err(AppError::BadRequestErr(
//             "User already exists with same phone",
//         ));
//     }

//     Ok(())
// }

// // check if the given email already exists in the users collection
// async fn check_uniq_email(client: DbInterface, email: &str) -> Result<(), AppError> {
//     let user_coll = &client.database(DB_NAME).collection::<Document>(COLL_USERS);
//     let result = user_coll.find_one(doc! {"email": email}, None).await?;
//     if result.is_some() {
//         return Err(AppError::BadRequestErr(
//             "User already exists with same email",
//         ));
//     }

//     Ok(())
// }

// // Generate and send otp
// async fn generate_send_otp(user_id: u32, client: DbInterface) -> anyhow::Result<()> {
//     let database = client.database(DB_NAME);
//     let user_coll = database.collection::<Document>(COLL_USERS);
//     let f = doc! {"id": user_id};
//     let user = user_coll
//         .find_one(f, None)
//         .await?
//         .ok_or(anyhow!("User not found with id: {user_id}"))?;
//     let phone = user.get_str("phone")?;
//     let otp = generate_otp(OTP_LENGTH);
//     let data = OtpSchema::new(user_id, otp.as_str());
//     let otp_coll = &database.collection::<OtpSchema>(COLL_OTP);
//     otp_coll.insert_one(data, None).await?;
//     send_otp(phone, &otp);
//     Ok(())
// }

// #[derive(Debug, Serialize)]
// #[serde(rename_all = "camelCase")]
// struct OtpSchema {
//     user_id: u32,
//     otp: String,
//     valid_till: u64,
//     is_used: bool,
//     update_ts: u64,
// }

// impl OtpSchema {
//     fn new(user_id: u32, otp: &str) -> Self {
//         Self {
//             user_id,
//             otp: otp.to_string(),
//             valid_till: get_epoch_ts() + OTP_VALIDITY_MINS * 60,
//             is_used: false,
//             update_ts: get_epoch_ts(),
//         }
//     }
// }

// fn send_otp(phone: &str, otp: &str) {
//     tracing::debug!("Send otp {otp} to phone {phone}");
// }
