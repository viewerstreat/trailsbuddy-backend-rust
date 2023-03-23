use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    utils::{error_handler::AppError, get_user_id_from_token},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

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
    params: Query<Params>,
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
fn create_find_by_doc(params: &Query<Params>) -> Result<Document, AppError> {
    let mut find_by = doc! {"isActive": true};
    if let Some(id) = &params.id {
        let oid = ObjectId::parse_str(id).map_err(|err| {
            tracing::debug!("Unable to parse _id params: {:?}", err);
            AppError::BadRequestErr("Unable to parse _id".into())
        })?;
        find_by.insert("_id", oid);
    }
    Ok(find_by)
}

fn pipeline_query(params: &Query<Params>, user_id: u32) -> Result<Vec<Document>, AppError> {
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

// #[cfg(test)]
// mod tests {
//     use axum::{
//         body::Body,
//         http::{Request, StatusCode},
//         routing::get,
//         Router,
//     };
//     use mockall::predicate::{eq, function};
//     use mongodb::{bson::doc, options::FindOptions};
//     use tower::ServiceExt;

//     use super::*;

//     fn get_test_clips() -> Vec<Clips> {
//         let mut clips = vec![];
//         let clip = Clips {
//             _id: ObjectId::new().to_hex(),
//             name: Some("Clip 1".to_string()),
//             description: None,
//             banner_image_url: None,
//             video_url: None,
//             view_count: Some(0),
//             like_count: Some(0),
//             is_active: true,
//             created_by: None,
//             created_ts: None,
//             updated_by: None,
//             updated_ts: None,
//         };
//         clips.push(clip);
//         let clip = Clips {
//             _id: ObjectId::new().to_hex(),
//             name: Some("Clip 1".to_string()),
//             description: None,
//             banner_image_url: None,
//             video_url: None,
//             view_count: Some(0),
//             like_count: Some(0),
//             is_active: true,
//             created_by: None,
//             created_ts: None,
//             updated_by: None,
//             updated_ts: None,
//         };
//         clips.push(clip);

//         clips
//     }

//     fn check_sort() -> Box<dyn Fn(&Document) -> Option<()>> {
//         let closure = |sort: &Document| {
//             if sort.iter().count() > 1 {
//                 return None;
//             }
//             sort.get_i32("_id")
//                 .ok()
//                 .and_then(|val| if val == -1 { Some(()) } else { None })
//         };
//         Box::new(closure)
//     }

//     fn check_limit(limit: u64) -> Box<dyn Fn(i64) -> Option<()>> {
//         let closure = move |val: i64| {
//             if val == limit as i64 {
//                 Some(())
//             } else {
//                 None
//             }
//         };
//         Box::new(closure)
//     }

//     fn check_skip(skip: u64) -> Box<dyn Fn(u64) -> Option<()>> {
//         let closure = move |val: u64| {
//             if val == skip {
//                 Some(())
//             } else {
//                 None
//             }
//         };
//         Box::new(closure)
//     }

//     #[tokio::test]
//     async fn test_get_clips_handler() {
//         let clips = get_test_clips();
//         let filter = Some(doc! {"isActive": true});
//         let check_options = function(|options: &Option<FindOptions>| {
//             options
//                 .as_ref()
//                 .and_then(|option| {
//                     option
//                         .sort
//                         .as_ref()
//                         .and_then(check_sort())
//                         .and(option.limit)
//                         .and_then(check_limit(DEFAULT_QUERY_LIMIT))
//                         .and(option.skip)
//                         .and_then(check_skip(0))
//                 })
//                 .is_some()
//         });
//         let mut mock_db = AppDatabase::default();
//         {
//             let clips = clips.clone();
//             mock_db
//                 .expect_find::<Clips>()
//                 .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
//                 .times(1)
//                 .returning(move |_, _, _, _| Ok(clips.clone()));
//         }
//         let db = Arc::new(mock_db);
//         let app = Router::new()
//             .route("/", get(get_clips_handler))
//             .with_state(db);
//         let req = Request::builder().uri("/").body(Body::empty()).unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         assert_eq!(res.status(), StatusCode::OK);
//         let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let response: Response = serde_json::from_slice(&bd).unwrap();
//         assert_eq!(response.success, true);
//         assert_eq!(response.data.len(), clips.len());
//         assert_eq!(response.data[0], clips[0]);
//         assert_eq!(response.data[1], clips[1]);
//     }

//     #[tokio::test]
//     async fn test_get_clips_handler_pagination() {
//         let clips = get_test_clips();
//         let page_index = 1u64;
//         let page_size = 10u64;
//         let filter = Some(doc! {"isActive": true});
//         let check_options = function(move |options: &Option<FindOptions>| {
//             options
//                 .as_ref()
//                 .and_then(|option| {
//                     option
//                         .sort
//                         .as_ref()
//                         .and_then(check_sort())
//                         .and(option.limit)
//                         .and_then(check_limit(page_size))
//                         .and(option.skip)
//                         .and_then(check_skip(page_size * page_index))
//                 })
//                 .is_some()
//         });
//         let mut mock_db = AppDatabase::default();
//         {
//             let clips = clips.clone();
//             mock_db
//                 .expect_find::<Clips>()
//                 .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
//                 .times(1)
//                 .returning(move |_, _, _, _| Ok(clips.clone()));
//         }
//         let db = Arc::new(mock_db);
//         let app = Router::new()
//             .route("/", get(get_clips_handler))
//             .with_state(db);
//         let uri = format!("/?pageSize={}&pageIndex={}", page_size, page_index);
//         let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         assert_eq!(res.status(), StatusCode::OK);
//         let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let response: Response = serde_json::from_slice(&bd).unwrap();
//         assert_eq!(response.success, true);
//         assert_eq!(response.data.len(), clips.len());
//         assert_eq!(response.data[0], clips[0]);
//         assert_eq!(response.data[1], clips[1]);
//     }

//     #[tokio::test]
//     async fn test_get_clips_by_id() {
//         let page_index = 1u64;
//         let page_size = 10u64;
//         let clips = get_test_clips();
//         let clip = clips[0].clone();
//         let clip_id = clip._id.clone();
//         let oid = ObjectId::parse_str(clip_id.clone()).unwrap();
//         let filter = Some(doc! {"isActive": true, "_id": oid});
//         let check_options = function(move |options: &Option<FindOptions>| {
//             options
//                 .as_ref()
//                 .and_then(|option| {
//                     option
//                         .sort
//                         .as_ref()
//                         .and_then(check_sort())
//                         .and(option.limit)
//                         .and_then(check_limit(page_size))
//                         .and(option.skip)
//                         .and_then(check_skip(0))
//                 })
//                 .is_some()
//         });
//         let mut mock_db = AppDatabase::default();
//         {
//             let clip = clip.clone();
//             mock_db
//                 .expect_find::<Clips>()
//                 .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
//                 .times(1)
//                 .returning(move |_, _, _, _| Ok(vec![clip.clone()]));
//         }
//         let db = Arc::new(mock_db);
//         let app = Router::new()
//             .route("/", get(get_clips_handler))
//             .with_state(db);
//         let uri = format!(
//             "/?_id={}&pageSize={}&pageIndex={}",
//             clip_id.as_str(),
//             page_size,
//             page_index
//         );
//         let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         assert_eq!(res.status(), StatusCode::OK);
//         let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let response: Response = serde_json::from_slice(&bd).unwrap();
//         assert_eq!(response.success, true);
//         assert_eq!(response.data.len(), 1);
//         assert_eq!(response.data[0], clip);
//     }
// }
