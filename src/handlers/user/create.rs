use axum::{extract::State, http::StatusCode, Json};
use mongodb::bson::{doc, Document};
use std::sync::Arc;

use super::otp::generate_send_otp;
use crate::{
    constants::*,
    database::AppDatabase,
    models::*,
    utils::{generate_referral_code, get_epoch_ts, get_seq_nxt_val, AppError, ValidatedBody},
};

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

/// User create
///
/// User signup for new user
#[utoipa::path(
    post,
    path = "/api/v1/user/create",
    request_body = CreateUserReq,
    responses(
        (status = StatusCode::CREATED, description = "User created successfully", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn create_user_handler(
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateUserReq>,
) -> Result<(StatusCode, Json<GenericResponse>), AppError> {
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
    let response = GenericResponse {
        success: true,
        message: "User created".to_string(),
    };
    let response = (StatusCode::CREATED, Json(response));
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
