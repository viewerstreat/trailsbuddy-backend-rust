use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::{doc, oid::ObjectId, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};
use validator::Validate;

use crate::{
    constants::*,
    handlers::contest::create::ContestStatus,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

#[double]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtraMediaType {
    Image,
    Video,
}

impl Display for ExtraMediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Image => write!(f, "image"),
            Self::Video => write!(f, "video"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    #[validate(range(min = 1, max = 4))]
    pub option_id: u32,
    #[validate(length(min = 1, max = 100))]
    pub option_text: String,
    pub is_correct: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_media_type: Option<ExtraMediaType>,
    #[validate(url)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_media_link: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub contest_id: String,
    pub question_no: u32,
    pub question_text: String,
    pub options: Vec<Answer>,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_media_type: Option<ExtraMediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_media_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_ts: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<u32>,
}

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
    let contest_id = ObjectId::parse_str(&body.contest_id).map_err(|err| {
        tracing::debug!("not able to parse contest_id: {:?}", err);
        AppError::BadRequestErr("not able to parse contestId".into())
    })?;
    if body.extra_media_type.is_some() && body.extra_media_link.is_none() {
        let err = AppError::BadRequestErr("extraMediaLink missing".into());
        return Err(err);
    }
    validate_options(&body.options)?;
    let (contest_check, duplicate_check) = tokio::join!(
        check_valid_contest(&db, &contest_id),
        check_duplicate_ques_no(&db, &contest_id, claims.id)
    );
    let _ = contest_check?;
    let _ = duplicate_check?;
    let question = Question {
        contest_id: body.contest_id,
        question_no: body.question_no,
        question_text: body.question_text,
        options: body.options,
        extra_media_type: body.extra_media_type,
        extra_media_link: body.extra_media_link,
        is_active: true,
        created_ts: Some(get_epoch_ts()),
        created_by: Some(claims.id),
        updated_ts: None,
        updated_by: None,
    };
    let _r = db
        .insert_one::<Question>(DB_NAME, COLL_QUESTIONS, &question, None)
        .await?;
    let res = json!({"success": true, "message": "Inserted successfully"});
    Ok(Json(res))
}

pub async fn check_valid_contest(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
) -> Result<(), AppError> {
    let filter = doc! {
        "_id": contest_id,
        "isActive": true,
        // "status": ContestStatus::CREATED.to_string()
    };
    let _result = db
        .find_one::<Document>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::BadRequestErr("Not valid contest".into()))?;
    Ok(())
}

async fn check_duplicate_ques_no(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    question_no: u32,
) -> Result<(), AppError> {
    let filter = doc! {"contestId": contest_id, "questionNo": question_no};
    let result = db
        .find_one::<Document>(DB_NAME, COLL_QUESTIONS, Some(filter), None)
        .await?;
    if result.is_some() {
        let err = AppError::BadRequestErr("Duplicate question".into());
        return Err(err);
    }
    Ok(())
}

fn validate_options(options: &Vec<Answer>) -> Result<(), AppError> {
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
