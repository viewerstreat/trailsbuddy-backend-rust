use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    contest::{AnswerProps, Question, QuestionProps},
    wallet::Money,
};
use crate::utils::get_epoch_ts;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestionWithoutCorrectFlag {
    #[serde(flatten)]
    pub props: QuestionProps,
    pub options: Vec<AnswerProps>,
    pub is_active: bool,
}

impl From<&Question> for QuestionWithoutCorrectFlag {
    fn from(value: &Question) -> Self {
        let options = value
            .options
            .iter()
            .map(|v| v.props.clone())
            .collect::<Vec<_>>();
        Self {
            props: value.props.clone(),
            options,
            is_active: value.is_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[allow(non_camel_case_types)]
pub enum PlayTrackerStatus {
    INIT,
    PAID,
    STARTED,
    FINISHED,
    ENDED,
}

impl PlayTrackerStatus {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChosenAnswer {
    #[serde(flatten)]
    pub question: Question,
    pub selected_option_id: u32,
}

impl ChosenAnswer {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayTracker {
    pub user_id: u32,
    pub contest_id: String,
    pub status: PlayTrackerStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_ts: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_questions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answers: Option<Vec<ChosenAnswer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_taken: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<u32>,
}

impl PlayTracker {
    pub fn new(user_id: u32, contest_id: &str, total_questions: u32) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            contest_id: contest_id.to_string(),
            status: PlayTrackerStatus::INIT,
            init_ts: Some(ts),
            start_ts: None,
            finish_ts: None,
            resume_ts: None,
            paid_ts: None,
            wallet_transaction_id: None,
            paid_amount: None,
            total_questions: Some(total_questions),
            score: None,
            answers: None,
            time_taken: None,
            rank: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }

    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}
