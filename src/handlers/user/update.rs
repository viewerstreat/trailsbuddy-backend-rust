use axum::{extract::State, Json};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use super::create::{check_uniq_email, check_uniq_phone};
use crate::{
    constants::*,
    jwt::JwtClaims,
    models::user::User,
    utils::{get_epoch_ts, validate_phonenumber, AppError, ValidatedBody},
};

use crate::database::AppDatabase;

#[derive(Debug, Default, Clone, Deserialize, Validate)]
pub struct UpdateUserReq {
    #[validate(length(min = 1, max = 50))]
    name: Option<String>,

    #[validate(custom(function = "validate_phonenumber"))]
    phone: Option<String>,

    #[validate(email)]
    email: Option<String>,

    #[serde(rename = "profilePic")]
    #[validate(url)]
    profile_pic: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateResponse {
    success: bool,
    data: User,
}

pub async fn update_user_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<UpdateUserReq>,
) -> Result<Json<UpdateResponse>, AppError> {
    // bad request if all params are none
    if body.name.is_none()
        && body.phone.is_none()
        && body.email.is_none()
        && body.profile_pic.is_none()
    {
        let err = AppError::BadRequestErr("name/phone/email/profilePic is required".into());
        return Err(err);
    }
    // check if phone already exists in the DB
    if let Some(phone) = &body.phone {
        check_uniq_phone(&db, phone).await?;
    }
    // check if email already exists in the DB
    if let Some(email) = &body.email {
        check_uniq_email(&db, email).await?;
    }
    let filter = doc! {"id": claims.id};
    let ts = get_epoch_ts() as i64;
    let mut set_obj = doc! {"updatedTs": ts};
    if let Some(name) = &body.name {
        set_obj.insert("name", name);
    }
    if let Some(phone) = &body.phone {
        set_obj.insert("phone", phone);
    }
    if let Some(email) = &body.email {
        set_obj.insert("email", email);
    }
    if let Some(profile_pic) = &body.profile_pic {
        set_obj.insert("profile_pic ", profile_pic);
    }
    let update = doc! {"$set": set_obj};
    let mut options = FindOneAndUpdateOptions::default();
    options.return_document = Some(ReturnDocument::After);
    let options = Some(options);
    let user = db
        .find_one_and_update::<User>(DB_NAME, COLL_USERS, filter, update, options)
        .await?
        .ok_or(AppError::NotFound("User not found".into()))?;

    let res = UpdateResponse {
        success: true,
        data: user,
    };
    Ok(Json(res))
}
