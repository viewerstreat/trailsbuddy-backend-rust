use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{parse_object_id, AppError},
};

/// get Question
#[utoipa::path(
    get,
    path = "/api/v1/question",
    params(ContestActivateReqBody, ("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "question list", body = GetQuestionResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn get_question_handler(
    _claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    params: Query<ContestActivateReqBody>,
) -> Result<Json<GetQuestionResponse>, AppError> {
    let contest_id = parse_object_id(&params.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<ContestWithQuestion>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?
        .ok_or(AppError::NotFound("Contest not found".into()))?;
    let data = contest
        .questions
        .and_then(|questions| Some(questions.into_iter().filter(|q| q.is_active).collect()));
    let res = GetQuestionResponse {
        success: true,
        data,
    };
    Ok(Json(res))
}
