use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::doc;
use mongodb::bson::serde_helpers::hex_string_as_object_id;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

use crate::handlers::question::create::Question;
use crate::{
    constants::*,
    jwt::JwtClaims,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

#[double]
use crate::database::AppDatabase;

use super::create::ContestStatus;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    #[serde(deserialize_with = "hex_string_as_object_id::deserialize")]
    #[serde(rename = "_id")]
    _id: String,
    start_time: u64,
    end_time: u64,
    status: ContestStatus,
    questions: Option<Vec<Question>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    contest_id: String,
}

pub async fn activate_contest_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter.clone()), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;
    if contest.status != ContestStatus::CREATED && contest.status != ContestStatus::INACTIVE {
        let err = "contest status must be CREATED or INACTIVE";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let ts = get_epoch_ts();
    if contest.end_time <= ts {
        let err = "contest is ended already";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let question_count = contest
        .questions
        .and_then(|questions| {
            let count = questions.iter().filter(|q| q.is_active).count();
            Some(count)
        })
        .unwrap_or(0);

    if question_count == 0 {
        let err = "contest should contain some questions";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let update = doc! {
        "$set": {
            "status": ContestStatus::ACTIVE.to_bson()?,
            "updatedBy": claims.id,
            "updatedTs": ts as i64
        }
    };
    db.update_one(DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;

    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

pub async fn inactivate_contest_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter.clone()), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;
    if contest.status != ContestStatus::CREATED && contest.status != ContestStatus::ACTIVE {
        let err = "contest status must be CREATED or ACTIVE";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let ts = get_epoch_ts();
    if contest.start_time <= ts {
        let err = "contest is started already";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let update = doc! {
        "$set": {
            "status": ContestStatus::INACTIVE.to_bson()?,
            "updatedBy": claims.id,
            "updatedTs": ts as i64
        }
    };
    db.update_one(DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;

    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}
