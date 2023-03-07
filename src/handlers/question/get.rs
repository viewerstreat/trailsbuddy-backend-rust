use axum::{
    extract::{Query, State},
    Json,
};
use mockall_double::double;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::create::Question;
use crate::{constants::*, jwt::JwtClaims, utils::AppError};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    contest_id: String,
    question_no: u32,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Question,
}

pub async fn get_question_handler(
    _claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let filter = doc! {"contestId": &params.contest_id, "questionNo": params.question_no};
    let data = db
        .find_one::<Question>(DB_NAME, COLL_QUESTIONS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Question not found".into()))?;
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}
