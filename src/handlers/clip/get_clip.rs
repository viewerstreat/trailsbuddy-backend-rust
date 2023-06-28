use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    utils::{error_handler::AppError, get_user_id_from_token, parse_object_id},
};

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Vec<Document>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    #[serde(rename = "_id")]
    id: Option<String>,
    page_index: Option<u64>,
    page_size: Option<u64>,
}

pub async fn get_clips_handler(
    headers: HeaderMap,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<Params>,
) -> Result<Json<Response>, AppError> {
    let user_id = get_user_id_from_token(&headers);
    let pipeline = pipeline_query(&params, user_id.unwrap_or_default())?;
    let data = db.aggregate(DB_NAME, COLL_CLIPS, pipeline, None).await?;
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}

// dynamic find_by filter doc based on the query params
fn create_find_by_doc(params: &Params) -> Result<Document, AppError> {
    let mut find_by = doc! {"isActive": true};
    if let Some(id) = &params.id {
        let oid = parse_object_id(id, "Unable to parse _id")?;
        find_by.insert("_id", oid);
    }
    Ok(find_by)
}

fn pipeline_query(params: &Params, user_id: u32) -> Result<Vec<Document>, AppError> {
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

#[cfg(test)]
mod tests {

    use mongodb::bson::oid::ObjectId;

    use super::*;

    #[test]
    fn test_create_find_by_doc() {
        let mut params = Params {
            id: None,
            page_size: None,
            page_index: None,
        };
        let doc = create_find_by_doc(&params).unwrap();
        assert_eq!(doc.contains_key("_id"), false);
        assert_eq!(doc.get_bool("isActive").unwrap(), true);

        params.id = Some("abcd".to_owned());
        let doc = create_find_by_doc(&params);
        assert!(doc.is_err());

        params.id = Some("6496fa0971e1a7e60d5a238b".to_owned());
        let doc = create_find_by_doc(&params).unwrap();
        assert_eq!(
            doc.get_object_id("_id").unwrap(),
            ObjectId::parse_str("6496fa0971e1a7e60d5a238b").unwrap()
        );
        assert_eq!(doc.get_bool("isActive").unwrap(), true);
    }

    #[test]
    fn test_pipeline_query() {
        let projection = doc! {
            "name": 1,
            "description": 1,
            "bannerImageUrl": 1,
            "videoUrl": 1,
            "isActive": 1,
            "_id": {"$toString": "$_id"},
            "likeCount": {"$size": "$likes"},
            "viewCount": {"$size": "$views"},
            "isLikedByMe": {"$cond": [{"$gt": [{"$size": "$myLikes"}, 0]}, true, false]}
        };
        let mut params = Params {
            id: None,
            page_size: None,
            page_index: None,
        };
        let user_id = 1;
        let find_by = create_find_by_doc(&params).unwrap();
        let pipeline = pipeline_query(&params, user_id).unwrap();
        assert_eq!(pipeline.len(), 7);
        assert_eq!(pipeline[0].get_document("$match").unwrap(), &find_by);
        assert_eq!(
            pipeline[1]
                .get_document("$addFields")
                .unwrap()
                .contains_key("likes"),
            true
        );
        assert_eq!(
            pipeline[1]
                .get_document("$addFields")
                .unwrap()
                .contains_key("views"),
            true
        );
        assert_eq!(
            pipeline[2]
                .get_document("$addFields")
                .unwrap()
                .contains_key("myLikes"),
            true
        );
        assert_eq!(
            pipeline[3].get_document("$sort").unwrap(),
            &doc! {"_id": -1}
        );
        assert_eq!(pipeline[4].get_i64("$skip").unwrap(), 0);
        assert_eq!(
            pipeline[5].get_i64("$limit").unwrap(),
            DEFAULT_QUERY_LIMIT as i64
        );
        assert_eq!(pipeline[6].get_document("$project").unwrap(), &projection);

        params.page_index = Some(1);
        params.page_size = Some(25);
        let pipeline = pipeline_query(&params, user_id).unwrap();
        assert_eq!(pipeline[4].get_i64("$skip").unwrap(), 25);
        assert_eq!(pipeline[5].get_i64("$limit").unwrap(), 25);
    }
}
