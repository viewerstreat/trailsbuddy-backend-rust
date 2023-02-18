use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::FindOptions,
    Client,
};
use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::utils::deserialize_objectid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipSchema {
    #[serde(deserialize_with = "deserialize_objectid")]
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
    data: Vec<ClipSchema>,

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
    State(client): State<Client>,
    params: Query<Params>,
) -> impl IntoResponse {
    // get the clips collection instance to query
    let coll = client
        .database(DB_NAME)
        .collection::<ClipSchema>(COLL_CLIPS);
    // calculate skip and limit value from query params pageIndex and pageSize
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let skip = page_index * page_size;
    // create the find_by filter doc
    let mut find_by = doc! {"isActive": true};
    if let Some(id) = &params.id {
        // create the ObjectId value from the string value
        let oid = ObjectId::parse_str(id);
        // return error respone when error
        if oid.is_err() {
            return send_error_response(oid.err().unwrap(), "not able to parse _id value");
        }
        // set the _id value in find_by document
        let oid = oid.unwrap();
        find_by.insert("_id", oid);
    }

    // create the document for sort by
    let sort_by = doc! {"_id": -1};
    // create the find options
    let mut options = FindOptions::default();
    options.sort = Some(sort_by);
    options.skip = Some(skip);
    options.limit = Some(page_size as i64);
    // get result from the database
    let result = coll.find(find_by, options).await;
    if result.is_err() {
        let err = result.err().unwrap();
        return send_error_response(err, "not able to fetch result from database");
    }
    let mut cursor = result.unwrap();
    let mut data: Vec<ClipSchema> = vec![];
    // loop through the cursor and collect result into data vec
    while let Some(doc) = cursor.next().await {
        if doc.is_err() {
            return send_error_response(doc.err().unwrap(), "Error in cursor read");
        }
        data.push(doc.unwrap());
    }
    // return successful response
    let res = Response {
        success: true,
        data,
        message: None,
    };
    (StatusCode::OK, Json(res))
}

// helper function to send Internal Server Error when mongo error occurred
fn send_error_response(
    err: impl std::error::Error,
    msg: &'static str,
) -> (StatusCode, Json<Response>) {
    tracing::debug!("{:?}", err);
    let res = Response {
        success: false,
        data: vec![],
        message: Some(msg),
    };
    (StatusCode::INTERNAL_SERVER_ERROR, Json(res))
}
