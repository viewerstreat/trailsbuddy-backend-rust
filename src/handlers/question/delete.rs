use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use super::update::{update_options_question, update_question};
use crate::{
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

/// Question delete
///
/// Delete a new question
#[utoipa::path(
    post,
    path = "/api/v1/question/delete",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = QuesDelReqBody,
    responses(
        (status = StatusCode::OK, description = "Question deleted", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn delete_question_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<QuesDelReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "_id": contest_id,
        "status": ContestStatus::CREATED.to_bson()?,
        "questions.questionNo": body.question_no
    };
    let update = doc! {
        "updatedTs": ts,
        "updatedBy": claims.id,
        "questions.$[elem].isActive": false
    };
    let update = doc! {"$set": update};
    let options = update_options_question(body.question_no);
    update_question(&db, filter, update, Some(options)).await?;
    let res = GenericResponse {
        success: true,
        message: "updated successfully".to_owned(),
    };
    Ok(Json(res))
}
