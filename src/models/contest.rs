use chrono::{prelude::*, serde::ts_seconds};
use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::utils::{deserialize_helper, get_epoch_ts, validation::validate_future_timestamp};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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
pub struct ContestProps {
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
    #[serde(with = "ts_seconds")]
    #[validate(custom = "validate_future_timestamp")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    #[validate(custom = "validate_future_timestamp")]
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub min_required_players: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_helper")]
    pub _id: Option<String>,
    #[serde(flatten)]
    pub props: ContestProps,
    pub status: ContestStatus,
    pub created_ts: Option<u64>,
    pub created_by: Option<u32>,
    pub updated_ts: Option<u64>,
    pub updated_by: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ContestWithQuestion {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_helper")]
    pub _id: Option<String>,
    #[serde(flatten)]
    pub props: ContestProps,
    pub status: ContestStatus,
    pub questions: Option<Vec<Question>>,
    pub created_ts: Option<u64>,
    pub created_by: Option<u32>,
    pub updated_ts: Option<u64>,
    pub updated_by: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnswerProps {
    #[validate(range(min = 1, max = 4))]
    pub option_id: u32,
    #[validate(length(min = 1, max = 100))]
    pub option_text: String,
    pub has_video: bool,
    pub has_image: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    pub image_or_video_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    #[serde(flatten)]
    #[validate]
    pub props: AnswerProps,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionProps {
    #[validate(range(min = 1))]
    pub question_no: u32,
    #[validate(length(min = 1, max = 200))]
    pub question_text: String,
    pub has_video: bool,
    pub has_image: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    pub image_or_video_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    #[serde(flatten)]
    #[validate]
    pub props: QuestionProps,
    #[validate]
    pub options: Vec<Answer>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionReqBody {
    #[serde(flatten)]
    #[validate]
    pub props: QuestionProps,
    #[validate]
    pub options: Vec<Answer>,
}

impl Contest {
    pub fn new(props: &ContestProps, user_id: u32) -> Self {
        let ts = get_epoch_ts();
        Self {
            _id: None,
            props: props.clone(),
            status: ContestStatus::CREATED,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }
}

impl Question {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

impl From<QuestionReqBody> for Question {
    fn from(value: QuestionReqBody) -> Self {
        Self {
            props: value.props,
            options: value.options,
            is_active: true,
        }
    }
}
