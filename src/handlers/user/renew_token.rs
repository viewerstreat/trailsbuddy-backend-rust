use axum::{extract::State, Json};
use mongodb::bson::{doc, Document};
use std::sync::Arc;

use super::login::{update_user_login, verify_fb_token, verify_id_token};
use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JWT_KEYS,
    models::*,
    utils::{get_epoch_ts, AppError},
};

type RetType = Result<Json<LoginResponse>, AppError>;

/// Renew token
///
/// Renew token using refreshToken/idToken/fbToken
#[utoipa::path(
    post,
    path = "/api/v1/user/renewToken",
    request_body = RenewTokenReqBody,
    responses(
        (status = StatusCode::OK, description = "Token renew successful", body = LoginResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn renew_token_handler(
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<RenewTokenReqBody>,
) -> RetType {
    match body.login_scheme {
        LoginScheme::GOOGLE => {
            let id_token = body
                .id_token
                .ok_or(AppError::BadRequestErr("idToken missing".into()))?;
            renew_token_google(&db, &id_token).await
        }
        LoginScheme::FACEBOOK => {
            let fb_token = body
                .fb_token
                .ok_or(AppError::BadRequestErr("fbToken missing".into()))?;
            renew_token_fb(&db, &fb_token).await
        }
        LoginScheme::OTP_BASED => {
            let refresh_token = body
                .refresh_token
                .ok_or(AppError::BadRequestErr("refreshToken missing".into()))?;
            renew_token_otp(&db, &refresh_token).await
        }
    }
}

async fn renew_token_google(db: &Arc<AppDatabase>, id_token: &str) -> RetType {
    let token_data = verify_id_token(id_token).await?;
    let filter = Some(doc! {"email": token_data.email, "isActive": true});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(AppError::BadRequestErr("user not found".into()))?;
    if LoginScheme::GOOGLE != user.login_scheme {
        let err = AppError::BadRequestErr("GOOGLE loginScheme was not used previously".into());
        return Err(err);
    };
    let user = update_user_login(db, user.id, user.login_scheme).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_owned()))?;
    let res = LoginResponse {
        success: true,
        data: user,
        token,
        refresh_token: None,
    };
    Ok(Json(res))
}

async fn renew_token_fb(db: &Arc<AppDatabase>, fb_token: &str) -> RetType {
    let token_data = verify_fb_token(fb_token).await?;
    let filter = Some(doc! {"email": token_data.email, "isActive": true});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(AppError::BadRequestErr("user not found".into()))?;
    if LoginScheme::FACEBOOK != user.login_scheme {
        let err = AppError::BadRequestErr("FACEBOOK loginScheme was not used previously".into());
        return Err(err);
    };
    let user = update_user_login(db, user.id, user.login_scheme).await?;
    let token = JWT_KEYS.generate_token(user.id, Some(user.name.to_owned()))?;
    let res = LoginResponse {
        success: true,
        data: user,
        token,
        refresh_token: None,
    };
    Ok(Json(res))
}

async fn renew_token_otp(db: &Arc<AppDatabase>, refresh_token: &str) -> RetType {
    let claims = JWT_KEYS
        .extract_claims(refresh_token)
        .ok_or(AppError::BadRequestErr("Invalid Token".into()))?;
    let user_id = claims.id;
    let filter = Some(doc! {"token": refresh_token});
    let data = db
        .find_one::<Document>(DB_NAME, COLL_USED_TOKENS, filter, None)
        .await?;
    if data.is_some() {
        let err = AppError::BadRequestErr("token already used".into());
        return Err(err);
    }
    let filter = Some(doc! {"id": user_id, "isActive": true});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?
        .ok_or(AppError::BadRequestErr("user not found".into()))?;
    if LoginScheme::OTP_BASED != user.login_scheme {
        let err = AppError::BadRequestErr("OTP_BASED loginScheme was not used previously".into());
        return Err(err);
    }
    let ts = get_epoch_ts() as i64;
    let document = doc! {"userId": user_id, "token": refresh_token, "updateTs": ts};
    db.insert_one::<Document>(DB_NAME, COLL_USED_TOKENS, &document, None)
        .await?;
    let token = JWT_KEYS.generate_token(user_id, Some(user.name.to_owned()))?;
    let refresh_token = JWT_KEYS.generate_refresh_token(user_id, None)?;
    let res = LoginResponse {
        success: true,
        data: user,
        token,
        refresh_token: Some(refresh_token),
    };
    Ok(Json(res))
}
