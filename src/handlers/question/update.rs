use axum::{extract::State, Json};
use mongodb::{
    bson::{doc, Bson, Document},
    options::UpdateOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::create::validate_options;
use crate::{
    constants::*,
    jwt::JwtClaims,
    models::contest::{Answer, ContestStatus, ExtraMediaType},
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
    question_text: Option<String>,
    #[validate]
    options: Option<Vec<Answer>>,
    nullify_extra_media: Option<bool>,
    extra_media_type: Option<ExtraMediaType>,
    #[validate(url)]
    extra_media_link: Option<String>,
}

pub async fn update_question_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    if body.question_text.is_none()
        && body.options.is_none()
        && body.extra_media_type.is_none()
        && body.nullify_extra_media.is_none()
    {
        let err = AppError::BadRequestErr("Please provide a field to update".into());
        return Err(err);
    }
    if body.extra_media_type.is_some() && body.extra_media_link.is_none() {
        let err = AppError::BadRequestErr("extraMediaLink missing".into());
        return Err(err);
    }
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::CREATED.to_bson()?,
        "questions.questionNo": body.question_no
    };
    let mut update = doc! {
        "updatedTs": ts,
        "updatedBy": claims.id,
        "questions.$[elem].questionNo": body.question_no
    };
    if let Some(question_text) = &body.question_text {
        update.insert("questions.$[elem].questionText", question_text);
    }
    if let Some(options) = body.options.as_ref() {
        validate_options(options)?;
        let mut bson_options = vec![];
        for option in options {
            bson_options.push(option.to_bson()?);
        }
        update.insert("questions.$[elem].options", bson_options);
    }
    if let Some(extra_media_type) = body.extra_media_type.as_ref() {
        if let Some(extra_media_link) = body.extra_media_link.as_ref() {
            update.insert(
                "questions.$[elem].extraMediaType",
                extra_media_type.to_bson()?,
            );
            update.insert("questions.$[elem].extraMediaLink", extra_media_link);
        }
    }
    if let Some(nullify_extra_media) = body.nullify_extra_media {
        if nullify_extra_media {
            update.insert("questions.$[elem].extraMediaType", Bson::Null);
            update.insert("questions.$[elem].extraMediaLink", Bson::Null);
        }
    }
    let update = doc! {"$set": update};
    let options = update_options_question(body.question_no);
    update_question(&db, filter, update, Some(options)).await?;
    let res = json!({"success": true, "message": "updated successfully"});
    Ok(Json(res))
}

pub fn update_options_question(question_no: u32) -> UpdateOptions {
    let array_filters = vec![doc! {"elem.questionNo": question_no}];
    UpdateOptions::builder()
        .array_filters(Some(array_filters))
        .build()
}

pub async fn update_question(
    db: &Arc<AppDatabase>,
    filter: Document,
    update: Document,
    options: Option<UpdateOptions>,
) -> Result<(), AppError> {
    let result = db
        .update_one(DB_NAME, COLL_CONTESTS, filter, update, options)
        .await?;
    if result.matched_count == 0 {
        let err = AppError::NotFound("question not found".into());
        return Err(err);
    }
    if result.matched_count != result.modified_count {
        tracing::debug!("not able to update database properly: {:?}", result);
        let err = anyhow::anyhow!("not able to update database");
        return Err(AppError::AnyError(err));
    }
    Ok(())
}
