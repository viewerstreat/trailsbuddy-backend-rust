use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use mongodb::bson::{doc, Document};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    models::*,
    utils::{error_handler::AppError, get_epoch_ts, get_user_id_from_token, parse_object_id},
};

/// Get movies
///
/// Get list of movies
/// Authoriztion token is non mandatory
#[utoipa::path(
    get,
    path = "/api/v1/movie",
    params(GetClipParams),
    params(("authorization" = Option<String>, Header, description = "JWT token")),
    security((),("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "Movie list", body = GetClipResponse),
    ),
    tag = "App User API"
)]
pub async fn get_movie_handler(
    headers: HeaderMap,
    State(db): State<Arc<AppDatabase>>,
    params: Query<GetClipParams>,
) -> Result<Json<GetClipResponse>, AppError> {
    let user_id = get_user_id_from_token(&headers);
    let pipeline = pipeline_query(&params, user_id.unwrap_or_default())?;
    let data = db.aggregate(DB_NAME, COLL_MOVIES, pipeline, None).await?;
    let data = data.into_iter().map(|d| d.into()).collect();
    let res = GetClipResponse {
        success: true,
        data,
    };
    Ok(Json(res))
}

// dynamic find_by filter doc based on the query params
fn create_find_by_doc(params: &Query<GetClipParams>) -> Result<Document, AppError> {
    let ts = get_epoch_ts() as i64;
    let mut find_by = doc! {"isActive": true, "moviePromotionExpiry": {"$gt": ts}};
    if let Some(id) = &params.id {
        let oid = parse_object_id(id, "Unable to parse _id")?;
        find_by.insert("_id", oid);
    }
    Ok(find_by)
}

fn pipeline_query(params: &Query<GetClipParams>, user_id: u32) -> Result<Vec<Document>, AppError> {
    let find_by = create_find_by_doc(params)?;
    let likes_filter = doc! {
        "$filter": {
            "input": "$likes",
            "as": "likes",
            "cond": {"$eq":["$$likes.isRemoved", false]}
        }
    };
    let add_fields = doc! {
        "likes": {"$ifNull": [likes_filter, []]},
        "views": {"$ifNull": ["$views", []]}
    };
    let add_field_my_likes = doc! {
        "myLikes": {
            "$filter": {
                "input": "$likes",
                "as": "likes",
                "cond": {"$eq": ["$$likes.userId", user_id]}
            }
        }
    };
    let projection = doc! {
        "name": 1,
        "description": 1,
        "bannerImageUrl": 1,
        "videoUrl": 1,
        "isActive": 1,
        "tags": 1,
        "sponsoredBy": 1,
        "sponsoredByLogo": 1,
        "releaseDate": 1,
        "releaseOutlets": 1,
        "moviePromotionExpiry": 1,
        "_id": {"$toString": "$_id"},
        "likeCount": {"$size": "$likes"},
        "viewCount": {"$size": "$views"},
        "isLikedByMe": {"$cond": [{"$gt": [{"$size": "$myLikes"}, 0]}, true, false]}
    };
    let sort_by = doc! {"_id": -1};
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let mut skip = page_index * page_size;
    // when searched by id page_index will be reset to zero
    if params.id.is_some() {
        skip = 0;
    }
    let mut pipeline = vec![];
    pipeline.push(doc! {"$match": find_by});
    pipeline.push(doc! {"$addFields": add_fields});
    pipeline.push(doc! {"$addFields": add_field_my_likes });
    pipeline.push(doc! {"$sort": sort_by});
    pipeline.push(doc! {"$skip": skip as i64});
    pipeline.push(doc! {"$limit": page_size as i64});
    pipeline.push(doc! {"$project": projection});
    Ok(pipeline)
}
