use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

/// clear notification
#[utoipa::path(
    post,
    path = "/api/v1/notification/clear",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ClearNotiReq,
    responses(
        (status = StatusCode::OK, description = "cleared", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn clear_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ClearNotiReq>,
) -> Result<Json<GenericResponse>, AppError> {
    let oid = parse_object_id(body._id.as_str(), "not able to parse ObjectId")?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isCleared": false, "_id": oid};
    let update = doc! {"$set": {"isCleared": true, "updatedTs": ts}};
    db.update_one(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}

/// clear all notifications
#[utoipa::path(
    post,
    path = "/api/v1/notification/clearall",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "cleared", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn clear_all_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<GenericResponse>, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isCleared": false };
    let update = doc! {"$set": {"isCleared": true, "updatedTs": ts}};
    db.update_many(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}
