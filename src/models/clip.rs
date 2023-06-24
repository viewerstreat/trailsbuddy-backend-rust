use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use crate::utils::get_epoch_ts;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LikesEntry {
    pub user_id: u32,
    pub is_removed: bool,
    pub created_ts: Option<u64>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Clips {
    pub name: String,
    pub description: String,
    pub banner_image_url: String,
    pub video_url: String,
    pub likes: Option<Vec<LikesEntry>>,
    pub views: Option<Vec<ViewsEntry>>,
    pub is_active: bool,
    pub created_by: Option<u32>,
    pub created_ts: Option<u64>,
    pub updated_by: Option<u32>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClipRespData {
    #[serde(rename = "_id")]
    _id: String,
    name: String,
    description: String,
    banner_image_url: String,
    video_url: String,
    likes: Option<Vec<LikesEntry>>,
    views: Option<Vec<ViewsEntry>>,
    is_active: bool,
}

#[derive(Debug, Deserialize)]
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
    pub fn new(
        name: &str,
        desctiption: &str,
        banner_image_url: &str,
        video_url: &str,
        user_id: u32,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: desctiption.to_string(),
            banner_image_url: banner_image_url.to_string(),
            video_url: video_url.to_string(),
            likes: Some(vec![]),
            views: Some(vec![]),
            is_active: true,
            created_by: Some(user_id),
            created_ts: Some(get_epoch_ts()),
            updated_by: None,
            updated_ts: None,
        }
    }

    pub fn to_clip_resp_data(&self, clip_id: &str) -> ClipRespData {
        ClipRespData {
            _id: clip_id.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            banner_image_url: self.banner_image_url.to_string(),
            video_url: self.video_url.to_string(),
            likes: self.likes.clone(),
            views: self.views.clone(),
            is_active: self.is_active,
        }
    }
}
