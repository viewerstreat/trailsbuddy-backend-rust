use mongodb::bson::serde_helpers::hex_string_as_object_id;
use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::utils::deserialize_helper;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ContestStatus {
    CREATED,
    ACTIVE,
    INACTIVE,
    FINISHED,
    CANCELLED,
    ENDED,
}

impl ContestStatus {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ContestCategory {
    Movie,
    Others,
}

impl ContestCategory {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PrizeSelection {
    TOP_WINNERS,
    RATIO_BASED,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_helper")]
    #[serde(default)]
    pub _id: Option<String>,
    #[validate(length(min = 1))]
    pub title: String,
    pub category: ContestCategory,
    #[validate(length(min = 1))]
    pub movie_id: Option<String>,
    #[validate(length(min = 1))]
    pub sponsored_by: String,
    #[validate(url)]
    pub sponsored_by_logo: Option<String>,
    #[validate(url)]
    pub banner_image_url: String,
    #[validate(url)]
    pub video_url: String,
    pub entry_fee: u32,
    pub entry_fee_max_bonus_money: u32,
    pub prize_selection: PrizeSelection,
    #[validate(range(min = 1))]
    pub top_winners_count: Option<u32>,
    #[validate(range(min = 1))]
    pub prize_ratio_numerator: Option<u32>,
    #[validate(range(min = 1))]
    pub prize_ratio_denominator: Option<u32>,
    pub prize_value_real_money: u32,
    pub prize_value_bonus_money: u32,
    #[validate(range(min = 1))]
    pub start_time: u64,
    #[validate(range(min = 1))]
    pub end_time: u64,
    pub status: Option<ContestStatus>,
    pub created_ts: Option<u64>,
    pub created_by: Option<u32>,
    pub updated_ts: Option<u64>,
    pub updated_by: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtraMediaType {
    Image,
    Video,
}

impl ExtraMediaType {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    #[validate(range(min = 1, max = 4))]
    pub option_id: u32,
    #[validate(length(min = 1, max = 100))]
    pub option_text: String,
    pub is_correct: bool,
    pub extra_media_type: Option<ExtraMediaType>,
    #[validate(url)]
    pub extra_media_link: Option<String>,
}

impl Answer {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question_no: u32,
    pub question_text: String,
    pub options: Vec<Answer>,
    pub is_active: bool,
    pub extra_media_type: Option<ExtraMediaType>,
    pub extra_media_link: Option<String>,
}

impl Question {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Deserialize)]
pub struct QuestionContest {
    #[serde(deserialize_with = "hex_string_as_object_id::deserialize")]
    _id: String,
    status: ContestStatus,
    pub questions: Option<Vec<Question>>,
}
