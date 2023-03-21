use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::model::{Contest, PlayTracker, Question};
use crate::{
    constants::*,
    handlers::play_tracker::{get::validate_contest, model::PlayTrackerStatus},
    jwt::JwtClaims,
    utils::{get_epoch_ts, get_random_num, parse_object_id, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    contest_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    contest_id: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: PlayTracker,
    question: Question,
}

pub async fn start_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &contest_id, claims.id)
    );
    let contest = contest_result?;
    let play_tracker = play_tracker_result?;
    if play_tracker.status == PlayTrackerStatus::INIT && contest.entry_fee > 0 {
        let err = "contest not paid yet";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let play_tracker = update_play_tracker(&db, &contest_id, claims.id, &play_tracker).await?;
    let question = get_question(&contest, &play_tracker)?;
    let res = Response {
        success: true,
        data: play_tracker,
        question,
    };

    Ok(Json(res))
}

pub async fn get_next_ques_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<Params>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&params.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &contest_id, claims.id)
    );
    let contest = contest_result?;
    let play_tracker = play_tracker_result?;
    let question = get_question(&contest, &play_tracker)?;
    let res = Response {
        success: true,
        data: play_tracker,
        question,
    };

    Ok(Json(res))
}

fn get_question(contest: &Contest, play_tracker: &PlayTracker) -> Result<Question, AppError> {
    let answered_questions = play_tracker
        .answers
        .as_ref()
        .and_then(|ans| {
            Some(
                ans.iter()
                    .map(|q| q.question.question_no)
                    .collect::<Vec<u32>>(),
            )
        })
        .unwrap_or(vec![]);
    let all_questions = contest
        .questions
        .as_ref()
        .ok_or(AppError::BadRequestErr("questions not found".into()))?;
    let all_questions = all_questions
        .into_iter()
        .filter(|q| q.is_active)
        .collect::<Vec<_>>();
    let total_question = all_questions.len();
    let is_answered =
        |q: &&Question| -> bool { answered_questions.iter().any(|&ans| ans == q.question_no) };
    if answered_questions.len() == total_question || all_questions.iter().all(is_answered) {
        let err = "all questions answered already";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let random_start = get_random_num(0, total_question);
    let question = all_questions
        .into_iter()
        .cycle()
        .skip(random_start)
        .skip_while(is_answered)
        .take(1)
        .cloned()
        .next()
        .ok_or(AppError::unknown_error())?;
    Ok(question)
}

pub async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": {"$ne": PlayTrackerStatus::FINISHED.to_bson()?}
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
    play_tracker: &PlayTracker,
) -> Result<PlayTracker, AppError> {
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": {"$ne": PlayTrackerStatus::FINISHED.to_bson()?}
    };
    let ts = get_epoch_ts() as i64;
    let mut update = doc! {"updatedTs": ts, "updatedBy": user_id};
    let update = match play_tracker.status {
        PlayTrackerStatus::INIT | PlayTrackerStatus::PAID => {
            update.insert("startTs", ts);
            update.insert("status", PlayTrackerStatus::STARTED.to_bson()?);
            doc! {"$set": update}
        }
        PlayTrackerStatus::STARTED => {
            doc! {
                "$push": {"resumeTs": ts},
                "$set": update
            }
        }
        _ => {
            let err = "playTracker is not in correct status";
            let err = AppError::BadRequestErr(err.into());
            return Err(err);
        }
    };

    let options = FindOneAndUpdateOptions::builder()
        .return_document(Some(ReturnDocument::After))
        .build();
    let options = Some(options);
    let play_tracker = db
        .find_one_and_update::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, filter, update, options)
        .await?
        .ok_or(AppError::AnyError(anyhow::anyhow!(
            "not able to update playTracker"
        )))?;
    Ok(play_tracker)
}
