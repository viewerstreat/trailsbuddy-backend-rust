use axum::{extract::State, Json};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::play_tracker::{answer::check_play_tracker, get::validate_contest},
    jwt::JwtClaims,
    models::play_tracker::{PlayTracker, PlayTrackerStatus},
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

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
        check_play_tracker(&db, &body.contest_id, claims.id)
    );
    let _contest = contest_result?;
    let _play_tracker = play_tracker_result?;
    let play_tracker = update_play_tracker(&db, &body.contest_id, claims.id).await?;
    let res = Response {
        success: true,
        data: play_tracker,
    };

    Ok(Json(res))
}

async fn update_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &str,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": PlayTrackerStatus::STARTED.to_bson()?,
    };
    let update = doc! {
        "$set": {
            "status": PlayTrackerStatus::FINISHED.to_bson()?,
            "finishTs": ts,
            "updatedTs": ts,
            "updatedBy": user_id
        }
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
