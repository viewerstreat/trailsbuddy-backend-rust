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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    message: Option<&'static str>,
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
    let skip = page_index * page_size;
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

    fn get_clips() -> Vec<Clips> {
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

    #[tokio::test]
    async fn test_get_clips_handler() {
        let clips = get_clips();
        let filter = Some(doc! {"isActive": true});
        let check_options = function(|options: &Option<FindOptions>| {
            options
                .as_ref()
                .and_then(|option| {
                    option
                        .sort
                        .as_ref()
                        .and_then(|sort| {
                            if sort.iter().count() > 1 {
                                return None;
                            }
                            sort.get_i32("_id").ok().and_then(|val| {
                                if val == -1 {
                                    Some(())
                                } else {
                                    None
                                }
                            })
                        })
                        .and(option.limit)
                        .and_then(|limit| {
                            if limit == DEFAULT_QUERY_LIMIT as i64 {
                                Some(())
                            } else {
                                None
                            }
                        })
                        .and(option.skip)
                        .and_then(|skip| if skip == 0 { Some(()) } else { None })
                })
                .is_some()
        });
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find::<Clips>()
            .with(eq(DB_NAME), eq(COLL_CLIPS), eq(filter), check_options)
            .times(1)
            .returning(move |_, _, _, _| Ok(clips.clone()));
        let db = Arc::new(mock_db);
        let app = Router::new()
            .route("/", get(get_clips_handler))
            .with_state(db);
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
