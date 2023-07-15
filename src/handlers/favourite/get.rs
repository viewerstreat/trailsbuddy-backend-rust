use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::{doc, Document};
use std::sync::Arc;

use crate::{
    constants::*, database::AppDatabase, jwt::JwtClaims, models::*, utils::error_handler::AppError,
};

/// Get favourite
#[utoipa::path(
    get,
    path = "/api/v1/favourite",
    params(GetFavParams, ("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "Successful", body = GetClipResponse),
    ),
    tag = "App User API"
)]
pub async fn get_favourite_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<GetFavParams>,
) -> Result<Json<GetClipResponse>, AppError> {
    let coll = media_collection(&params.media_type);
    let pipeline = pipeline_query(&params, claims.id);
    let data = db.aggregate(DB_NAME, coll, pipeline, None).await?;
    let data = data.into_iter().map(|v| v.into()).collect();
    let res = GetClipResponse {
        success: true,
        data,
    };
    Ok(Json(res))
}

fn media_collection(media_type: &MediaType) -> &str {
    match media_type {
        MediaType::Clip => COLL_CLIPS,
        MediaType::Movie => COLL_MOVIES,
    }
}

fn pipeline_query(params: &GetFavParams, user_id: u32) -> Vec<Document> {
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
        "mediaType": &params.media_type,
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
