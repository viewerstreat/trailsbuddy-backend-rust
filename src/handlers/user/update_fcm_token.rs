use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

/// Update fcmToken for an user
#[utoipa::path(
    post,
    path = "/api/v1/user/updateFcmToken",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = FcmTokenReqBody,
    responses(
        (status = StatusCode::OK, description = "FCM token saved", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn update_fcm_token_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<FcmTokenReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let filter = doc! {"id": claims.id};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter.clone()), None)
        .await?;
    let fcm_tokens = user.and_then(|user| user.fcm_tokens);
    if let Some(token) = fcm_tokens {
        if token
            .iter()
            .any(|token| token.as_str() == body.token.as_str())
        {
            let res = GenericResponse {
                success: true,
                message: "token already exists for user".to_owned(),
            };
            return Ok(Json(res));
        }
    }
    let ts = get_epoch_ts() as i64;
    let update = doc! {"$set": {"updatedTs": ts}, "$push": {"fcmTokens": &body.token}};
    let result = db
        .update_one(DB_NAME, COLL_USERS, filter, update, None)
        .await?;
    if result.matched_count < 1 || result.modified_count < 1 {
        let err = anyhow::anyhow!("not able to update user");
        return Err(AppError::AnyError(err));
    }
    let res = GenericResponse {
        success: true,
        message: "token saved successfully".to_owned(),
    };
    Ok(Json(res))
}
