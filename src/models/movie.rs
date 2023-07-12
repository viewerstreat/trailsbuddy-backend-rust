use mongodb::bson::serde_helpers::hex_string_as_object_id;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::clip::{LikesEntry, ViewsEntry};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieProps {
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    #[serde(flatten)]
    pub props: MovieProps,
    pub created_by: Option<u32>,
    pub updated_by: Option<u32>,
    pub created_ts: Option<u64>,
    pub updated_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieRespData {
    #[serde(rename = "_id")]
    _id: String,
    #[serde(flatten)]
    props: MovieProps,
}

impl Movie {
    pub fn to_movie_resp_data(&self, movie_id: &str) -> MovieRespData {
        MovieRespData {
            _id: movie_id.to_string(),
            props: self.props.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieDetails {
    #[serde(rename = "_id")]
    #[serde(deserialize_with = "hex_string_as_object_id::deserialize")]
    pub id: String,
    #[serde(flatten)]
    pub props: MovieProps,
    pub view_count: Option<u32>,
    pub like_count: Option<u32>,
}
