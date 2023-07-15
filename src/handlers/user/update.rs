use axum::{extract::State, Json};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use std::sync::Arc;

use super::create::{check_uniq_email, check_uniq_phone};
use crate::database::AppDatabase;
use crate::{
    constants::*,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

/// Update user
///
/// Update name, phone, email or profilePic field in user details
/// phone and email fields are checked for unique value if provided
#[utoipa::path(
    post,
    path = "/api/v1/user/update",
    request_body = UpdateUserReq,
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "Update successful", body = UpdateUserResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn update_user_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<UpdateUserReq>,
) -> Result<Json<UpdateUserResponse>, AppError> {
    // bad request if all params are none
    if body.name.is_none()
        && body.phone.is_none()
        && body.email.is_none()
        && body.profile_pic.is_none()
    {
        let err = "name/phone/email/profilePic is required";
        let err = AppError::BadRequestErr(err.into());
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

    let res = UpdateUserResponse {
        success: true,
        data: user,
    };
    Ok(Json(res))
}
