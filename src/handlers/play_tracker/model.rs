use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use crate::{
    handlers::{contest::create::ContestStatus, question::create::ExtraMediaType},
    utils::get_epoch_ts,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    pub title: String,
    pub entry_fee: u32,
    pub entry_fee_max_bonus_money: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub status: Option<ContestStatus>,
    pub questions: Option<Vec<Question>>,
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



#[derive(Debug, Clone,Deserialize, Serialize )]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub option_id: u32,
    pub option_text: String,
    pub extra_media_type: Option<ExtraMediaType>,
    pub extra_media_link: Option<String>,
}






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
    pub question: Question,
    pub selected_option_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayTracker {
    pub user_id: u32,
    pub contest_id: String,
    pub status: PlayTrackerStatus,
    pub init_ts: Option<u64>,
    pub start_ts: Option<u64>,
    pub finish_ts: Option<u64>,
    pub resume_ts: Option<Vec<u64>>,
    pub paid_ts: Option<u64>,
    pub wallet_transaction_id: Option<String>,
    pub total_questions: usize,
    pub total_answered: usize,
    pub score: Option<u32>,
    pub curr_question_no: Option<u32>,
    pub answers: Option<Vec<GivenAnswer>>,
    pub time_taken: Option<u32>,
    pub rank: Option<u32>,
    pub created_ts: Option<u64>,
    pub created_by: Option<u32>,
    pub updated_ts: Option<u64>,
    pub updated_by: Option<u32>,
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
