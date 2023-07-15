use axum::{extract::State, Json};
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
};
use std::sync::Arc;

use super::create::validate_request;
use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

/// Question update
///
/// Update a new question
#[utoipa::path(
    post,
    path = "/api/v1/question/update",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = CreateQuesReqBody,
    responses(
        (status = StatusCode::OK, description = "Question updated", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn update_question_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateQuesReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    validate_request(&db, &body, &contest_id, false).await?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::CREATED.to_bson()?,
        "questions.questionNo": body.question.props.question_no
    };
    let question: Question = body.question.into();
    let update = doc! {
        "$set": {
            "updatedTs": ts,
            "updatedBy": claims.id,
            "questions.$[elem]": question.to_bson()?
        }
    };

    let options = update_options_question(question.props.question_no);
    update_question(&db, filter, update, Some(options)).await?;
    let res = GenericResponse {
        success: true,
        message: "updated successfully".to_owned(),
    };
    Ok(Json(res))
}

pub fn update_options_question(question_no: u32) -> UpdateOptions {
    let array_filters = vec![doc! {"elem.questionNo": question_no}];
    UpdateOptions::builder()
        .array_filters(Some(array_filters))
        .build()
}

pub async fn update_question(
    db: &Arc<AppDatabase>,
    filter: Document,
    update: Document,
    options: Option<UpdateOptions>,
) -> Result<(), AppError> {
    let result = db
        .update_one(DB_NAME, COLL_CONTESTS, filter, update, options)
        .await?;
    if result.matched_count == 0 {
        let err = AppError::NotFound("question not found".into());
        return Err(err);
    }
    if result.matched_count != result.modified_count {
        tracing::debug!("not able to update database properly: {:?}", result);
        let err = anyhow::anyhow!("not able to update database");
        return Err(AppError::AnyError(err));
    }
    Ok(())
}
