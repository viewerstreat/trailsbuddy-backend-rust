use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use crate::{handlers::question::create::Question, utils::get_epoch_ts};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GivenAnswer {
    #[serde(flatten)]
    question: Question,
    selected_option_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayTracker {
    user_id: u32,
    contest_id: String,
    status: PlayTrackerStatus,
    init_ts: Option<u64>,
    start_ts: Option<u64>,
    finish_ts: Option<u64>,
    resume_ts: Option<Vec<u64>>,
    paid_ts: Option<u64>,
    wallet_transaction_id: Option<String>,
    total_questions: usize,
    total_answered: usize,
    score: Option<u32>,
    curr_question_no: Option<u32>,
    answers: Option<Vec<GivenAnswer>>,
    time_taken: Option<u32>,
    rank: Option<u32>,
    created_ts: Option<u64>,
    created_by: Option<u32>,
    updated_ts: Option<u64>,
    updated_by: Option<u32>,
}

impl PlayTracker {
    pub fn new(user_id: u32, contest_id: &str, total_questions: usize) -> Self {
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
            total_questions,
            total_answered: 0,
            score: None,
            curr_question_no: None,
            answers: None,
            time_taken: None,
            rank: None,
            created_ts: Some(ts),
            created_by: Some(user_id),
            updated_ts: None,
            updated_by: None,
        }
    }
}
