use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
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
}

pub async fn delete_question_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = ObjectId::parse_str(&body.contest_id).map_err(|err| {
        tracing::debug!("not able to parse contest_id: {:?}", err);
        AppError::BadRequestErr("not able to parse contestId".into())
    })?;
    // check_valid_contest(&db, &contest_id).await?;
    let filter = doc! {
        "contestId": &body.contest_id,
        "questionNo": body.question_no,
        "isActive": true
    };
    let ts = get_epoch_ts() as i64;
    let update = doc! {"$set": {"isActive": false, "updatedBy": claims.id, "updatedTs": ts}};
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
    let res = json!({"success": true, "message": "deleted successfully"});
    Ok(Json(res))
}
