use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    jwt::JwtClaims,
    models::contest::{Question, QuestionContest},
    utils::{parse_object_id, AppError},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    contest_id: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Option<Vec<Question>>,
}

pub async fn get_question_handler(
    _claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&params.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<QuestionContest>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Contest not found".into()))?;
    let data = contest
        .questions
        .and_then(|questions| Some(questions.into_iter().filter(|q| q.is_active).collect()));
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}
