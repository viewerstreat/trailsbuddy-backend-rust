use serde::{Deserialize, Serialize};

use crate::handlers::clip::model::{LikesEntry, ViewsEntry};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub name: String,
    pub description: String,
    pub tags: Option<Vec<String>>,
    pub video_url: String,
    pub banner_image_url: String,
    pub sponsored_by: Option<String>,
    pub sponsored_by_logo: Option<String>,
    pub release_date: Option<u64>,
    pub release_outlets: Option<Vec<String>>,
    pub movie_promotion_expiry: Option<u64>,
    pub likes: Option<Vec<LikesEntry>>,
    pub views: Option<Vec<ViewsEntry>>,
    pub is_active: bool,
    pub created_by: Option<u32>,
    pub updated_by: Option<u32>,
    pub created_ts: Option<u64>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MovieRespData {
    #[serde(rename = "_id")]
    _id: String,
    name: String,
    description: String,
    tags: Option<Vec<String>>,
    banner_image_url: String,
    video_url: String,
    likes: Option<Vec<LikesEntry>>,
    views: Option<Vec<ViewsEntry>>,
    sponsored_by: Option<String>,
    sponsored_by_logo: Option<String>,
    release_date: Option<u64>,
    release_outlets: Option<Vec<String>>,
    movie_promotion_expiry: Option<u64>,
    is_active: bool,
}

impl Movie {
    pub fn to_movie_resp_data(&self, movie_id: &str) -> MovieRespData {
        MovieRespData {
            _id: movie_id.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            tags: self.tags.clone(),
            banner_image_url: self.banner_image_url.to_string(),
            video_url: self.video_url.to_string(),
            likes: self.likes.clone(),
            views: self.views.clone(),
            sponsored_by: self.sponsored_by.clone(),
            sponsored_by_logo: self.sponsored_by_logo.clone(),
            release_date: self.release_date.clone(),
            release_outlets: self.release_outlets.clone(),
            movie_promotion_expiry: self.movie_promotion_expiry.clone(),
            is_active: self.is_active,
        }
    }
}
