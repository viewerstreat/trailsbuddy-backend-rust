use mongodb::bson::{doc, Bson};
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
