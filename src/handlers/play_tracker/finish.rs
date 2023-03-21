use axum::{extract::State, Json};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use super::model::PlayTracker;
use crate::{
    constants::*,
    handlers::{
        contest::create::ContestStatus, play_tracker::model::PlayTrackerStatus,
        question::create::Contest,
    },
    jwt::JwtClaims,
    utils::{get_epoch_ts, get_random_num, parse_object_id, AppError, ValidatedBody},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    #[validate(length(min = 1))]
    contest_id: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: PlayTracker,
}

pub async fn finish_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &contest_id, claims.id)
    );
    let _contest = contest_result?;
    let _play_tracker = play_tracker_result?;
    let play_tracker = update_play_tracker(&db, &contest_id, claims.id).await?;
    let res = Response {
        success: true,
        data: play_tracker,
    };

    Ok(Json(res))
}

async fn validate_contest(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
) -> Result<Contest, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::ACTIVE.to_bson()?,
        "startTime": {"$lt": ts },
        "endTime": {"$gt": ts}
    };
    let contest = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;
    Ok(contest)
}

async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": PlayTrackerStatus::STARTED.to_bson()?,
    };
    let play_tracker = db
        .find_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Play Tracker not found".into()))?;
    Ok(play_tracker)
}

async fn update_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": PlayTrackerStatus::STARTED.to_bson()?,
    };
    let update = doc! {
        "status": PlayTrackerStatus::FINISHED.to_bson()?,
        "finishTs": ts,
        "updatedTs": ts,
        "updatedBy": user_id
    };
    let options = FindOneAndUpdateOptions::builder()
        .return_document(Some(ReturnDocument::After))
        .build();
    let play_tracker = db
        .find_one_and_update::<PlayTracker>(
            DB_NAME,
            COLL_PLAY_TRACKERS,
            filter,
            update,
            Some(options),
        )
        .await?
        .ok_or(AppError::unknown_error())?;

    Ok(play_tracker)
}
