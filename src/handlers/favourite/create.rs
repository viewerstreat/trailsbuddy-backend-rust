use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::{doc, oid::ObjectId};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::{
    fmt::{self, Display},
    sync::Arc,
};

#[double]
use crate::database::AppDatabase;
use crate::{
    constants::*,
    handlers::clip::model::LikesEntry,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Clip,
    Movie,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Clip => write!(f, "clip"),
            Self::Movie => write!(f, "movie"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    media_type: MediaType,
    media_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    _id: ObjectId,
    likes: Option<Vec<LikesEntry>>,
}

pub async fn add_favourite_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let coll = match body.media_type {
        MediaType::Clip => COLL_CLIPS,
        MediaType::Movie => COLL_MOVIES,
    };
    let oid = ObjectId::parse_str(body.media_id.as_str()).map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("not able to parse mediaId".into())
    })?;
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
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}
