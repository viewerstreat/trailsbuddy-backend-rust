use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use mockall_double::double;
use mongodb::{
    bson::{doc, Document},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use validator::Validate;

use crate::{
    constants::*,
    utils::{
        generate_otp, get_epoch_ts, get_seq_nxt_val, validate_phonenumber, AppError, ValidatedBody,
    },
};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LoginScheme {
    #[default]
    OTP_BASED,
    GOOGLE,
    FACEBOOK,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: u32,
    name: String,
    phone: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    profile_pic: Option<String>,

    login_scheme: LoginScheme,
    is_active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_login_time: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    has_used_referral_code: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    referral_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    referred_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    total_played: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    contest_won: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    total_earning: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    created_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    updated_ts: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Otp {
    user_id: u32,
    otp: String,
    valid_till: u64,
    is_used: bool,
    update_ts: u64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
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

impl Otp {
    fn new(user_id: u32, otp: &str) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            otp: otp.to_string(),
            valid_till: ts + OTP_VALIDITY_MINS * 60,
            is_used: false,
            update_ts: ts,
        }
    }
}

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

// check if the given phone already exists in users collection
async fn check_uniq_phone(db: &Arc<AppDatabase>, phone: &str) -> Result<(), AppError> {
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
async fn check_uniq_email(db: &Arc<AppDatabase>, email: &str) -> Result<(), AppError> {
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

// Generate and send otp
async fn generate_send_otp(user_id: u32, db: &Arc<AppDatabase>) -> anyhow::Result<()> {
    let f = Some(doc! {"id": user_id});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, f, None)
        .await?
        .ok_or(anyhow!("User not found with id: {user_id}"))?;
    let otp = generate_otp(OTP_LENGTH);
    let otp = Otp::new(user_id, otp.as_str());
    db.insert_one::<Otp>(DB_NAME, COLL_OTP, &otp, None).await?;
    send_otp(&user.phone, &otp.otp);
    Ok(())
}

// send otp to a given phone. SMS gateway API or SMS queue API to be called from here
fn send_otp(phone: &str, otp: &str) {
    tracing::debug!("Send otp {otp} to phone {phone}");
}

#[cfg(test)]
mod tests {
    use mockall::predicate::{eq, function};
    use mongodb::options::FindOneOptions;

    use super::*;

    #[tokio::test]
    async fn test_check_uniq_phone() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_phone(&db, phone).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_phone_exists() {
        let phone = "1234567890";
        let filter = Some(doc! {"phone": phone});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_phone(&db, phone).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same phone: {}", phone);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
        }
    }

    #[tokio::test]
    async fn test_check_uniq_email() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let db = Arc::new(mock_db);
        let _ = check_uniq_email(&db, email).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_uniq_email_exists() {
        let email = "testemail@email.com";
        let filter = Some(doc! {"email": email});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<Document>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(filter), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(doc! {"id": 1})));
        let db = Arc::new(mock_db);
        let result = check_uniq_email(&db, email).await;
        assert_eq!(result.is_err(), true);
        let msg = format!("User already exists with same email: {}", email);
        let result = result.err().unwrap();
        if let AppError::BadRequestErr(err) = result {
            assert_eq!(err, msg);
        } else {
            panic!("AppError::BadRequestErr should be received");
        }
    }
}

#[cfg(test)]
mod otp_tests {

    use mockall::predicate::{eq, function};
    use mongodb::options::{FindOneOptions, InsertOneOptions};

    use super::*;

    #[test]
    fn test_new() {
        let user_id = 1;
        let otp_val = "887797";
        let otp = Otp::new(user_id, otp_val);
        assert_eq!(otp.user_id, user_id);
        assert_eq!(otp.otp, otp_val);
        assert_eq!(otp.valid_till, otp.update_ts + OTP_VALIDITY_MINS * 60);
        assert_eq!(otp.is_used, false);
    }

    #[tokio::test]
    async fn test_generate_send_otp() {
        let user_id = 1;
        let f = Some(doc! {"id": user_id});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let check_none_ins = function(|options: &Option<InsertOneOptions>| options.is_none());
        let check_otp = function(move |otp: &Otp| {
            otp.user_id == user_id
                && otp.is_used == false
                && otp.otp.len() == OTP_LENGTH as usize
                && otp.valid_till > get_epoch_ts()
        });
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<User>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(f), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(User::default())));

        mock_db
            .expect_insert_one::<Otp>()
            .with(eq(DB_NAME), eq(COLL_OTP), check_otp, check_none_ins)
            .times(1)
            .returning(|_, _, _, _| Ok(String::new()));
        let db = Arc::new(mock_db);
        let result = generate_send_otp(user_id, &db).await;
        assert_eq!(result.is_ok(), true);
    }
}
