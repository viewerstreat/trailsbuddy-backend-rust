use axum::{extract::State, Json};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::play_tracker::{answer::check_play_tracker, get::validate_contest},
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

/// finish play tracker
#[utoipa::path(
    post,
    path = "/api/v1/playTracker/finish",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ContestIdRequest,
    responses(
        (status = StatusCode::OK, description = "finish PlayTracker", body = PlayTrackerResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn finish_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ContestIdRequest>,
) -> Result<Json<PlayTrackerResponse>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &body.contest_id, claims.id)
    );
    let _contest = contest_result?;
    let _play_tracker = play_tracker_result?;
    let play_tracker = update_play_tracker(&db, &body.contest_id, claims.id).await?;
    let res = PlayTrackerResponse {
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
