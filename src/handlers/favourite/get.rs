use axum::{
    extract::{Query, State},
    Json,
};
use mockall_double::double;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::create::MediaType;
use crate::{constants::*, jwt::JwtClaims, utils::error_handler::AppError};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Vec<Document>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    media_type: MediaType,
    page_index: Option<u64>,
    page_size: Option<u64>,
}

pub async fn get_favourite_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let coll = match params.media_type {
        MediaType::Clip => COLL_CLIPS,
        MediaType::Movie => COLL_MOVIES,
    };
    let pipeline = pipeline_query(&params, claims.id);
    let data = db.aggregate(DB_NAME, coll, pipeline, None).await?;
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}

fn pipeline_query(params: &Params, user_id: u32) -> Vec<Document> {
    let find_by = doc! {
        "isActive": true,
        "likes": {"$elemMatch": {"userId": user_id, "isRemoved": false}}
    };
    let sort_by = doc! {"_id": -1};
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let skip = page_index * page_size;
    let add_fields = doc! {
        "mediaId": {"$toString": "$_id"},
        "mediaType": params.media_type.to_string(),
        "mediaName": "$name",
        "userId": user_id
    };
    let projection = doc! {
      "_id": 0,
      "mediaId": 1,
      "mediaName": 1,
      "mediaType": 1,
      "userId": 1,
      "bannerImageUrl": 1
    };
    let mut pipeline = vec![];
    pipeline.push(doc! {"$match": find_by});
    pipeline.push(doc! {"$addFields": add_fields});
    pipeline.push(doc! {"$sort": sort_by});
    pipeline.push(doc! {"$skip": skip as i64});
    pipeline.push(doc! {"$limit": page_size as i64});
    pipeline.push(doc! {"$project": projection});
    pipeline
}
