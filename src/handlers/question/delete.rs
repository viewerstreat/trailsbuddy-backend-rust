use axum::{extract::State, Json};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use crate::{
    handlers::contest::create::ContestStatus,
    jwt::JwtClaims,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

use super::update::{update_options_question, update_question};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    #[validate(length(min = 1))]
    contest_id: String,
    #[validate(range(min = 1))]
    question_no: u32,
}

pub async fn delete_question_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::CREATED.to_bson()?,
        "questions.questionNo": body.question_no
    };
    let update = doc! {
        "updatedTs": ts,
        "updatedBy": claims.id,
        "questions.$[elem].isActive": false
    };
    let update = doc! {"$set": update};
    let options = update_options_question(body.question_no);
    update_question(&db, filter, update, Some(options)).await?;
    let res = json!({"success": true, "message": "updated successfully"});
    Ok(Json(res))
}
