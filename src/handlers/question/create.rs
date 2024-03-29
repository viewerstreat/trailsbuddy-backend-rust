use axum::{extract::State, Json};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

/// Question create
///
/// Create a new question
#[utoipa::path(
    post,
    path = "/api/v1/question",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = CreateQuesReqBody,
    responses(
        (status = StatusCode::OK, description = "Contest created", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn create_question_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateQuesReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    validate_request(&db, &body, &contest_id, true).await?;
    let question: Question = body.question.into();
    let ts = get_epoch_ts() as i64;
    let filter = doc! { "_id": contest_id };
    let update = doc! {
        "$push": {"questions": question.to_bson()?},
        "$set": {"updatedTs": ts, "updatedBy": claims.id}
    };
    db.update_one(DB_NAME, COLL_CONTESTS, filter, update, None)
        .await?;
    let res = GenericResponse {
        success: true,
        message: "Inserted successfully".to_owned(),
    };
    Ok(Json(res))
}

pub async fn validate_request(
    db: &Arc<AppDatabase>,
    body: &CreateQuesReqBody,
    contest_id: &ObjectId,
    create_request: bool,
) -> Result<(), AppError> {
    if (body.question.props.has_image || body.question.props.has_video)
        && body.question.props.image_or_video_url.is_none()
    {
        let err = AppError::BadRequestErr("imageOrVideoUrl missing".into());
        return Err(err);
    }
    let filter = doc! {"_id": contest_id, "status": ContestStatus::CREATED.to_bson()?};
    let contest = db
        .find_one::<ContestWithQuestion>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Not valid contest".into()))?;
    if create_request {
        if let Some(questions) = contest.questions.as_ref() {
            if questions
                .iter()
                .any(|ques| ques.props.question_no == body.question.props.question_no)
            {
                let err = AppError::BadRequestErr("Duplicate question".into());
                return Err(err);
            }
        }
    }
    validate_options(body.question.options.as_ref())?;

    Ok(())
}

fn validate_options(options: &Vec<Answer>) -> Result<(), AppError> {
    if options.len() != 4 {
        let err = AppError::BadRequestErr("options array must have 4 values".into());
        return Err(err);
    }
    let correct_count = options.iter().filter(|ans| ans.is_correct).count();
    if correct_count != 1 {
        let err = AppError::BadRequestErr("options must have one correct answer".into());
        return Err(err);
    }
    if (1..4).any(|idx| {
        let option_id = options[idx - 1].props.option_id;
        options[idx..]
            .iter()
            .any(|opt| opt.props.option_id == option_id)
    }) {
        let err = AppError::BadRequestErr("Duplicate optionId".into());
        return Err(err);
    }
    if options.iter().any(|opt| {
        (opt.props.has_image || opt.props.has_video) && opt.props.image_or_video_url.is_none()
    }) {
        let err = AppError::BadRequestErr("imageOrVideoUrl missing in options".into());
        return Err(err);
    }
    Ok(())
}
