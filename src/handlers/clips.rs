use axum::{
    extract::{Query, State},
    Json,
};
use futures::stream::StreamExt;
use mongodb::bson::serde_helpers::hex_string_as_object_id;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::FindOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};

use crate::{constants::*, utils::error_handler::AppError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipSchema {
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
) -> Result<Json<Response>, AppError> {
    // get the clips collection instance to query
    let coll = client
        .database(DB_NAME)
        .collection::<ClipSchema>(COLL_CLIPS);
    let find_by = create_find_by_doc(&params)?;
    let options = create_find_options(&params);
    let data = get_query_result(coll, find_by, options).await?;
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

// query the database and return result
async fn get_query_result(
    coll: Collection<ClipSchema>,
    find_by: Document,
    options: FindOptions,
) -> anyhow::Result<Vec<ClipSchema>> {
    let mut cursor = coll.find(find_by, options).await?;
    let mut data: Vec<ClipSchema> = vec![];
    // loop through the cursor and collect result into data vec
    while let Some(doc) = cursor.next().await {
        data.push(doc?);
    }
    Ok(data)
}
