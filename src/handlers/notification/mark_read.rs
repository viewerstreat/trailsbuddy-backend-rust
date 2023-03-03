use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
pub struct MarkReadNotiReq {
    #[validate(length(equal = 24))]
    _id: String,
}

pub async fn mark_read_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<MarkReadNotiReq>,
) -> Result<Json<JsonValue>, AppError> {
    let oid = ObjectId::parse_str(body._id).map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("not able to parse ObjectId".into())
    })?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isRead": false, "_id": oid};
    let update = doc! {"$set": {"isRead": true, "updatedTs": ts}};
    db.update_one(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

pub async fn mark_all_read_noti_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<JsonValue>, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": claims.id, "isRead": false };
    let update = doc! {"$set": {"isRead": true, "updatedTs": ts}};
    db.update_many(DB_NAME, COLL_NOTIFICATIONS, filter, update, None)
        .await?;
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}
