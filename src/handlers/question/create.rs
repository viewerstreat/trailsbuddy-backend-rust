use axum::{extract::State, Json};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use crate::models::contest::{Answer, ContestStatus, ExtraMediaType, Question, QuestionContest};
use crate::{
    constants::*,
    jwt::JwtClaims,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    #[validate(length(min = 1))]
    contest_id: String,
    #[validate(range(min = 1))]
    question_no: u32,
    #[validate(length(min = 1, max = 200))]
    question_text: String,
    #[validate]
    options: Vec<Answer>,
    extra_media_type: Option<ExtraMediaType>,
    #[validate(url)]
    extra_media_link: Option<String>,
}

pub async fn create_question_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    validate_request(&db, &body, &contest_id).await?;
    let question = Question {
        question_no: body.question_no,
        question_text: body.question_text,
        options: body.options,
        extra_media_type: body.extra_media_type,
        extra_media_link: body.extra_media_link,
        is_active: true,
    };
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": contest_id};
    let update = doc! {
        "$push": {"questions": question.to_bson()?},
        "$set": {"updatedTs": ts, "updatedBy": claims.id}
    };
    db.update_one(DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;
    let res = json!({"success": true, "message": "Inserted successfully"});
    Ok(Json(res))
}

pub async fn validate_request(
    db: &Arc<AppDatabase>,
    body: &ReqBody,
    contest_id: &ObjectId,
) -> Result<(), AppError> {
    if body.extra_media_type.is_some() && body.extra_media_link.is_none() {
        let err = AppError::BadRequestErr("extraMediaLink missing".into());
        return Err(err);
    }
    let filter = doc! {"_id": contest_id, "status": ContestStatus::CREATED.to_bson()?};
    let contest = db
        .find_one::<QuestionContest>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Not valid contest".into()))?;
    if let Some(questions) = contest.questions.as_ref() {
        if questions
            .iter()
            .any(|ques| ques.question_no == body.question_no)
        {
            let err = AppError::BadRequestErr("Duplicate question".into());
            return Err(err);
        }
    }
    validate_options(body.options.as_ref())?;

    Ok(())
}

pub fn validate_options(options: &Vec<Answer>) -> Result<(), AppError> {
    if options.len() != 4 {
        let err = AppError::BadRequestErr("options array must have 4 values".into());
        return Err(err);
    }
    let correct_count = options.iter().filter(|ans| ans.is_correct).count();
    if correct_count != 1 {
        let err = AppError::BadRequestErr("options must have one correct answer".into());
        return Err(err);
    }
    if (1..4).any(|idx| {
        let option_id = options[idx - 1].option_id;
        options[idx..].iter().any(|opt| opt.option_id == option_id)
    }) {
        let err = AppError::BadRequestErr("Duplicate optionId".into());
        return Err(err);
    }
    if options
        .iter()
        .any(|opt| opt.extra_media_type.is_some() && opt.extra_media_link.is_none())
    {
        let err = AppError::BadRequestErr("extraMediaLink missing".into());
        return Err(err);
    }
    Ok(())
}
