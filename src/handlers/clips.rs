use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use mockall_double::double;
use mongodb::bson::serde_helpers::hex_string_as_object_id;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};

use crate::{constants::*, utils::error_handler::AppError};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clips {
    #[serde(deserialize_with = "hex_string_as_object_id::deserialize")]
    _id: String,

    name: Option<String>,
    description: Option<String>,

    #[serde(rename = "bannerImageUrl")]
    banner_image_url: Option<String>,

    #[serde(rename = "videoUrl")]
    video_url: Option<String>,

    #[serde(rename = "viewCount")]
    view_count: Option<u32>,

    #[serde(rename = "likeCount")]
    like_count: Option<u32>,

    #[serde(rename = "isActive")]
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    success: bool,
    data: Vec<Clips>,

    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
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
    State(app_db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let find_by = Some(create_find_by_doc(&params)?);
    let options = Some(create_find_options(&params));
    let data = app_db
        .find::<Clips>(DB_NAME, COLL_CLIPS, find_by, options)
        .await?;
    // return successful response
    let res = Response {
        success: true,
        data,
        message: None,
    };
    Ok(Json(res))
}

// dynamic find_by filter doc based on the query params
fn create_find_by_doc(params: &Query<Params>) -> anyhow::Result<Document> {
    // always filter by isActive = true
    let mut find_by = doc! {"isActive": true};
    // if query params contain _id value the include in the filter
    if let Some(id) = &params.id {
        // create the ObjectId value from the string value
        let oid = ObjectId::parse_str(id)?;
        find_by.insert("_id", oid);
    }
    Ok(find_by)
}

// create find options based on query params
fn create_find_options(params: &Query<Params>) -> FindOptions {
    // calculate skip and limit value from query params pageIndex and pageSize
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let mut skip = page_index * page_size;
    // when searched by id page_index will be reset to zero
    if params.id.is_some() {
        skip = 0;
    }
    // create sort_by doc
    let sort_by = doc! {"_id": -1};
    // create FindOptions
    let mut options = FindOptions::default();
    options.sort = Some(sort_by);
    options.skip = Some(skip);
    options.limit = Some(page_size as i64);
    options
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use mockall::predicate::{eq, function};
    use mongodb::{bson::doc, options::FindOptions};
    use tower::ServiceExt;

    use super::*;

    fn get_test_clips() -> Vec<Clips> {
        let mut clips = vec![];
        let clip = Clips {
            _id: ObjectId::new().to_hex(),
            name: Some("Clip 1".to_string()),
            description: None,
            banner_image_url: None,
            video_url: None,
            view_count: Some(0),
            like_count: Some(0),
            is_active: true,
        };
        clips.push(clip);
        let clip = Clips {
            _id: ObjectId::new().to_hex(),
            name: Some("Clip 1".to_string()),
            description: None,
            banner_image_url: None,
            video_url: None,
            view_count: Some(0),
            like_count: Some(0),
            is_active: true,
        };
        clips.push(clip);

        clips
    }

    fn check_sort() -> Box<dyn Fn(&Document) -> Option<()>> {
        let closure = |sort: &Document| {
            if sort.iter().count() > 1 {
                return None;
            }
            sort.get_i32("_id")
                .ok()
                .and_then(|val| if val == -1 { Some(()) } else { None })
        };
        Box::new(closure)
    }

    fn check_limit(limit: u64) -> Box<dyn Fn(i64) -> Option<()>> {
        let closure = move |val: i64| {
            if val == limit as i64 {
                Some(())
            } else {
                None
            }
        };
        Box::new(closure)
    }

    fn check_skip(skip: u64) -> Box<dyn Fn(u64) -> Option<()>> {
        let closure = move |val: u64| {
            if val == skip {
                Some(())
            } else {
                None
            }
        };
        Box::new(closure)
    }

    #[tokio::test]
    async fn test_get_clips_handler() {
        let clips = get_test_clips();
        let filter = Some(doc! {"isActive": true});
        let check_options = function(|options: &Option<FindOptions>| {
            options
                .as_ref()
                .and_then(|option| {
                    option
                        .sort
                        .as_ref()
                        .and_then(check_sort())
                        .and(option.limit)
                        .and_then(check_limit(DEFAULT_QUERY_LIMIT))
                        .and(option.skip)
                        .and_then(check_skip(0))
                })
                .is_some()
        });
        let mut mock_db = AppDatabase::default();
        {
            let clips = clips.clone();
            mock_db
                .expect_find::<Clips>()
                .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
                .times(1)
                .returning(move |_, _, _, _| Ok(clips.clone()));
        }
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", get(get_clips_handler))
            .with_state(db);
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.data.len(), clips.len());
        assert_eq!(response.data[0], clips[0]);
        assert_eq!(response.data[1], clips[1]);
    }

    #[tokio::test]
    async fn test_get_clips_handler_pagination() {
        let clips = get_test_clips();
        let page_index = 1u64;
        let page_size = 10u64;
        let filter = Some(doc! {"isActive": true});
        let check_options = function(move |options: &Option<FindOptions>| {
            options
                .as_ref()
                .and_then(|option| {
                    option
                        .sort
                        .as_ref()
                        .and_then(check_sort())
                        .and(option.limit)
                        .and_then(check_limit(page_size))
                        .and(option.skip)
                        .and_then(check_skip(page_size * page_index))
                })
                .is_some()
        });
        let mut mock_db = AppDatabase::default();
        {
            let clips = clips.clone();
            mock_db
                .expect_find::<Clips>()
                .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
                .times(1)
                .returning(move |_, _, _, _| Ok(clips.clone()));
        }
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", get(get_clips_handler))
            .with_state(db);
        let uri = format!("/?pageSize={}&pageIndex={}", page_size, page_index);
        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.data.len(), clips.len());
        assert_eq!(response.data[0], clips[0]);
        assert_eq!(response.data[1], clips[1]);
    }

    #[tokio::test]
    async fn test_get_clips_by_id() {
        let page_index = 1u64;
        let page_size = 10u64;
        let clips = get_test_clips();
        let clip = clips[0].clone();
        let clip_id = clip._id.clone();
        let oid = ObjectId::parse_str(clip_id.clone()).unwrap();
        let filter = Some(doc! {"isActive": true, "_id": oid});
        let check_options = function(move |options: &Option<FindOptions>| {
            options
                .as_ref()
                .and_then(|option| {
                    option
                        .sort
                        .as_ref()
                        .and_then(check_sort())
                        .and(option.limit)
                        .and_then(check_limit(page_size))
                        .and(option.skip)
                        .and_then(check_skip(0))
                })
                .is_some()
        });
        let mut mock_db = AppDatabase::default();
        {
            let clip = clip.clone();
            mock_db
                .expect_find::<Clips>()
                .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
                .times(1)
                .returning(move |_, _, _, _| Ok(vec![clip.clone()]));
        }
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", get(get_clips_handler))
            .with_state(db);
        let uri = format!(
            "/?_id={}&pageSize={}&pageIndex={}",
            clip_id.as_str(),
            page_size,
            page_index
        );
        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bd = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&bd).unwrap();
        assert_eq!(response.success, true);
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0], clip);
    }
}
