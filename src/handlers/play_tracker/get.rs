use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

/// get play tracker
#[utoipa::path(
    get,
    path = "/api/v1/playTracker",
    params(ContestIdRequest, ("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "Get PlayTracker", body = PlayTrackerResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn get_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    params: Query<ContestIdRequest>,
) -> Result<Json<PlayTrackerResponse>, AppError> {
    let contest_id = parse_object_id(&params.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &params.contest_id, claims.id)
    );
    if let Some(play_tracker) = play_tracker_result? {
        let res = PlayTrackerResponse {
            success: true,
            data: play_tracker,
        };
        return Ok(Json(res));
    }
    let _contest = contest_result?;
    let play_tracker = insert_new_play_tracker(claims.id, &params.contest_id, &db).await?;
    let res = PlayTrackerResponse {
        success: true,
        data: play_tracker,
    };
    Ok(Json(res))
}

pub async fn insert_new_play_tracker(
    user_id: u32,
    contest_id: &str,
    db: &Arc<AppDatabase>,
) -> Result<PlayTracker, AppError> {
    let play_tracker = PlayTracker::new(user_id, contest_id);
    db.insert_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, &play_tracker, None)
        .await?;
    Ok(play_tracker)
}

pub async fn validate_contest(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
) -> Result<ContestWithQuestion, AppError> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::ACTIVE.to_bson()?,
        "startTime": {"$lt": ts },
        "endTime": {"$gt": ts}
    };
    let contest = db
        .find_one::<ContestWithQuestion>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;

    Ok(contest)
}

pub async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &str,
    user_id: u32,
) -> Result<Option<PlayTracker>, AppError> {
    let filter = doc! {"contestId": contest_id, "userId": user_id};
    let play_tracker = db
        .find_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?;
    Ok(play_tracker)
}
