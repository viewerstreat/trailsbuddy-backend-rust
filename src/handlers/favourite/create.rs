use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

/// Add favourite
#[utoipa::path(
    post,
    path = "/api/v1/favourite",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = AddFavReqBody,
    responses(
        (status = StatusCode::OK, description = "Successful", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::NOT_FOUND, description = "Media not found", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn add_favourite_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<AddFavReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let coll = match body.media_type {
        MediaType::Clip => COLL_CLIPS,
        MediaType::Movie => COLL_MOVIES,
    };
    let oid = parse_object_id(&body.media_id, "not able to parse mediaId")?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": oid, "isActive": true};
    let media = db
        .find_one::<Media>(DB_NAME, coll, Some(filter.clone()), None)
        .await?
        .ok_or(AppError::NotFound("media not found".into()))?;
    let idx = media
        .likes
        .as_ref()
        .and_then(|likes| likes.iter().position(|like| like.user_id == claims.id));
    if let Some(idx) = idx {
        let is_removed = media.likes.unwrap()[idx].is_removed;
        let filter = doc! {"_id": oid, "isActive": true, "likes.userId": claims.id};
        let update = doc! {
            "$set": {
                "likes.$.isRemoved": !is_removed,
                "likes.$.updatedTs": ts,
                "updatedTs": ts,
                "updatedBy": claims.id
            }
        };
        db.update_one(DB_NAME, coll, filter, update, None).await?;
    } else {
        let update = doc! {
            "$push": {"likes": LikesEntry::new(claims.id)},
            "$set": {"updatedTs": ts, "updatedBy": claims.id}
        };
        db.update_one(DB_NAME, coll, filter, update, None).await?;
    }
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}
