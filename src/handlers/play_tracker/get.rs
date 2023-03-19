use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::model::{Contest, PlayTracker};
use crate::{
    constants::*,
    handlers::contest::create::ContestStatus,
    jwt::JwtClaims,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    contest_id: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: PlayTracker,
}

pub async fn get_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&params.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &contest_id, claims.id)
    );
    if let Some(play_tracker) = play_tracker_result? {
        let res = Response {
            success: true,
            data: play_tracker,
        };
        return Ok(Json(res));
    }
    let contest = contest_result?;
    let total_question = contest
        .questions
        .and_then(|questions| {
            let count = questions.iter().filter(|q| q.is_active).count();
            Some(count)
        })
        .unwrap_or(0);
    let play_tracker = PlayTracker::new(claims.id, &params.contest_id, total_question);
    db.insert_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, &play_tracker, None)
        .await?;
    let res = Response {
        success: true,
        data: play_tracker,
    };
    Ok(Json(res))
}

pub async fn validate_contest(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
) -> Result<Contest, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::ACTIVE.to_bson()?,
        "startTime": {"$gte": ts },
        "endTime": {"$lt": ts}
    };
    let contest = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;

    Ok(contest)
}

pub async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    user_id: u32,
) -> Result<Option<PlayTracker>, AppError> {
    let filter = doc! {"_id": contest_id, "userId": user_id};
    let play_tracker = db
        .find_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?;
    Ok(play_tracker)
}