use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::get_epoch_ts;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LikesEntry {
    pub user_id: u32,
    pub is_removed: bool,
    pub created_ts: Option<u64>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ViewsEntry {
    pub user_id: u32,
    pub updated_ts: Option<u64>,
}

impl LikesEntry {
    pub fn new(user_id: u32) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            is_removed: false,
            created_ts: Some(ts),
            updated_ts: None,
        }
    }
}

impl From<LikesEntry> for Bson {
    fn from(value: LikesEntry) -> Self {
        let created_ts = value.created_ts.and_then(|ts| Some(ts as i64));
        let updated_ts = value.updated_ts.and_then(|ts| Some(ts as i64));
        let d = doc! {"userId": value.user_id, "isRemoved": value.is_removed, "createdTs": created_ts, "updatedTs": updated_ts};
        Self::Document(d)
    }
}

impl From<ViewsEntry> for Bson {
    fn from(value: ViewsEntry) -> Self {
        let ts = value.updated_ts.and_then(|ts| Some(ts as i64));
        let d = doc! {"userId": value.user_id, "updatedTs": ts};
        Self::Document(d)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClipProps {
    pub name: String,
    pub description: String,
    pub banner_image_url: String,
    pub video_url: String,
    pub likes: Option<Vec<LikesEntry>>,
    pub views: Option<Vec<ViewsEntry>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clips {
    #[serde(flatten)]
    pub props: ClipProps,
    pub created_by: Option<u32>,
    pub created_ts: Option<u64>,
    pub updated_by: Option<u32>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClipRespData {
    #[serde(rename = "_id")]
    _id: String,
    #[serde(flatten)]
    pub props: ClipProps,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Clip,
    Movie,
}

impl From<&MediaType> for Bson {
    fn from(value: &MediaType) -> Self {
        match value {
            MediaType::Clip => Self::String("clip".to_owned()),
            MediaType::Movie => Self::String("movie".to_owned()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Media {
    _id: ObjectId,
    pub likes: Option<Vec<LikesEntry>>,
}

impl Clips {
    pub fn to_clip_resp_data(&self, clip_id: &str) -> ClipRespData {
        ClipRespData {
            _id: clip_id.to_string(),
            props: self.props.clone(),
        }
    }
}
