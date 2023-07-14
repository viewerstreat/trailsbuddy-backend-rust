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

/// mark notification as read
#[utoipa::path(
    post,
    path = "/api/v1/notification/markRead",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ClearNotiReq,
    responses(
        (status = StatusCode::OK, description = "marked as read", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn mark_read_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ClearNotiReq>,
) -> Result<Json<GenericResponse>, AppError> {
    let oid = parse_object_id(body._id.as_str(), "not able to parse ObjectId")?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isRead": false, "_id": oid};
    let update = doc! {"$set": {"isRead": true, "updatedTs": ts}};
    db.update_one(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}

/// mark all notification as read
#[utoipa::path(
    post,
    path = "/api/v1/notification/markAllRead",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "marked as read", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn mark_all_read_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<GenericResponse>, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isRead": false };
    let update = doc! {"$set": {"isRead": true, "updatedTs": ts}};
    db.update_many(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}
