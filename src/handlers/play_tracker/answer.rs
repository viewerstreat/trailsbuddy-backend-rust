use axum::{extract::State, Json};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use super::model::{GivenAnswer, PlayTracker, Question};
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
    #[validate(range(min = 1))]
    question_no: u32,
    #[validate(range(min = 1, max = 4))]
    selected_option_id: u32,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: PlayTracker,
    question: Option<Question>,
}

pub async fn answer_play_tracker_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &contest_id, claims.id)
    );
    let contest = contest_result?;
    let play_tracker = play_tracker_result?;
    let (given_answer, is_correct) = check_if_correct(&contest, &body)?;
    let is_finished = check_if_finished(&contest, &play_tracker)?;
    let play_tracker = update_play_tracker(
        &db,
        &contest_id,
        claims.id,
        is_correct,
        is_finished,
        &given_answer,
    )
    .await?;
    let question = get_question(&contest, &play_tracker, is_finished)?;
    let res = Response {
        success: true,
        data: play_tracker,
        question,
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

fn check_if_correct(contest: &Contest, body: &ReqBody) -> Result<(GivenAnswer, bool), AppError> {
    let questions = contest
        .questions
        .as_ref()
        .ok_or(AppError::BadRequestErr("questions not found".into()))?;
    let question = questions
        .into_iter()
        .find(|q| q.question_no == body.question_no)
        .ok_or(AppError::BadRequestErr("Invalid questionNo".into()))?;
    let is_correct = question
        .options
        .iter()
        .any(|opt| opt.option_id == body.selected_option_id);
    let answer = GivenAnswer {
        question: question.clone(),
        selected_option_id: body.selected_option_id,
    };
    Ok((answer, is_correct))
}

fn check_if_finished(contest: &Contest, play_tracker: &PlayTracker) -> Result<bool, AppError> {
    let questions = contest
        .questions
        .as_ref()
        .ok_or(AppError::BadRequestErr("questions not found".into()))?;
    let empty_vec = vec![];
    let answers = play_tracker.answers.as_ref().unwrap_or(&empty_vec);
    let is_finished = questions.into_iter().all(|q| {
        answers
            .into_iter()
            .any(|ans| ans.question.question_no == q.question_no)
    });

    Ok(is_finished)
}

async fn update_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &ObjectId,
    user_id: u32,
    is_correct: bool,
    is_finished: bool,
    answer: &GivenAnswer,
) -> Result<PlayTracker, AppError> {
    let score = if is_correct { 1 } else { 0u32 };
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
        "status": PlayTrackerStatus::STARTED.to_bson()?,
    };
    let mut set_obj = doc! {"updatedTs": ts, "updatedBy": user_id};
    if is_finished {
        set_obj.insert("finishTs", ts);
        set_obj.insert("status", PlayTrackerStatus::FINISHED.to_bson()?);
    }
    let update = doc! {
        "$push": {"answers": answer.to_bson()?},
        "$inc": {"score": score},
        "$set": set_obj
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

fn get_question(
    contest: &Contest,
    play_tracker: &PlayTracker,
    is_finished: bool,
) -> Result<Option<Question>, AppError> {
    if is_finished {
        return Ok(None);
    }
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
        .map(|q| q.into())
        .collect::<Vec<Question>>();
    let total_question = all_questions.len();
    let is_answered =
        |q: &Question| -> bool { answered_questions.iter().any(|&ans| ans == q.question_no) };
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
        .next()
        .ok_or(AppError::unknown_error())?;
    Ok(Some(question))
}
