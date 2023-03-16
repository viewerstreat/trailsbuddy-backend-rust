use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::{doc, oid::ObjectId, ser::to_bson, Document};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::create::{Answer, ExtraMediaType};
use crate::{
    constants::*,
    handlers::contest::create::ContestStatus,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

#[double]
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
    extra_media_type: Option<ExtraMediaType>,
    #[validate(url)]
    extra_media_link: Option<String>,
}

pub async fn update_question_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = ObjectId::parse_str(&body.contest_id).map_err(|err| {
        tracing::debug!("not able to parse contest_id: {:?}", err);
        AppError::BadRequestErr("not able to parse contestId".into())
    })?;
    if body.question_text.is_none() && body.options.is_none() && body.extra_media_type.is_none() {
        let err = AppError::BadRequestErr("Please provide a field to update".into());
        return Err(err);
    }
    if body.extra_media_type.is_some() && body.extra_media_link.is_none() {
        let err = AppError::BadRequestErr("extraMediaLink missing".into());
        return Err(err);
    }
    let filter = doc! {
        "contestId": &body.contest_id,
        "questionNo": body.question_no,
        "isActive": true
    };
    let ts = get_epoch_ts() as i64;
    let mut update = doc! {"updatedTs": ts, "updatedBy": claims.id};
    if let Some(question_text) = &body.question_text {
        update.insert("questionText", question_text);
    }
    if let Some(extra_media_type) = &body.extra_media_type {
        if let Some(extra_media_link) = &body.extra_media_link {
            // update.insert("extraMediaType", extra_media_type.to_string());
            update.insert("extraMediaLink", extra_media_link);
        }
    }
    if let Some(options) = &body.options {
        let options = to_bson(options).map_err(|err| {
            tracing::debug!("not able to convert options to bson: {:?}", err);
            let err = anyhow::anyhow!("not able to convert options to bson");
            AppError::AnyError(err)
        })?;
        update.insert("options", options);
    }
    let update = doc! {"$set": update};
    // let result = db
    //     .update_one(DB_NAME, COLL_QUESTIONS, filter, update, None)
    //     .await?;
    // if result.matched_count == 0 {
    //     let err = AppError::NotFound("question not found".into());
    //     return Err(err);
    // }
    // if result.matched_count != result.modified_count {
    //     tracing::debug!("not able to update database properly: {:?}", result);
    //     let err = anyhow::anyhow!("not able to update database");
    //     return Err(AppError::AnyError(err));
    // }
    let res = json!({"success": true, "message": "updated successfully"});
    Ok(Json(res))
}
