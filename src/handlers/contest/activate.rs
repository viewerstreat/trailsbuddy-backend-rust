use axum::{extract::State, Json};
use mongodb::bson::{doc, Document};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

/// Activate contest
#[utoipa::path(
    post,
    path = "/api/v1/contest/activate",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ContestIdRequest,
    responses(
        (status = StatusCode::OK, description = "Contest activated", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn activate_contest_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ContestIdRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<ContestWithQuestion>(DB_NAME, COLL_CONTESTS, Some(filter.clone()), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;
    if contest.status != ContestStatus::CREATED && contest.status != ContestStatus::INACTIVE {
        let err = "contest status must be CREATED or INACTIVE";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let ts = get_epoch_ts() as i64;
    if contest.props.end_time.timestamp() <= ts {
        let err = "contest is ended already";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let question_count = contest
        .questions
        .and_then(|questions| {
            let count = questions.iter().filter(|q| q.is_active).count();
            Some(count)
        })
        .unwrap_or(0);

    if question_count == 0 {
        let err = "contest should contain some questions";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let update = doc! {
        "$set": {
            "status": ContestStatus::ACTIVE.to_bson()?,
            "updatedBy": claims.id,
            "updatedTs": ts as i64
        }
    };
    update_contest(&db, filter, update).await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}

/// Inactivate contest
#[utoipa::path(
    post,
    path = "/api/v1/contest/inActivate",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ContestIdRequest,
    responses(
        (status = StatusCode::OK, description = "Contest inactivated", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn inactivate_contest_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ContestIdRequest>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let filter = doc! {"_id": contest_id};
    let contest = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter.clone()), None)
        .await?
        .ok_or(AppError::NotFound("contest not found".into()))?;
    if contest.status != ContestStatus::CREATED && contest.status != ContestStatus::ACTIVE {
        let err = "contest status must be CREATED or ACTIVE";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let ts = get_epoch_ts() as i64;
    if contest.props.start_time.timestamp() <= ts {
        let err = "contest is started already";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let update = doc! {
        "$set": {
            "status": ContestStatus::INACTIVE.to_bson()?,
            "updatedBy": claims.id,
            "updatedTs": ts as i64
        }
    };
    update_contest(&db, filter, update).await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}

async fn update_contest(
    db: &Arc<AppDatabase>,
    query: Document,
    update: Document,
) -> anyhow::Result<()> {
    db.update_one(DB_NAME, COLL_CONTESTS, query, update, None)
        .await?;
    Ok(())
}
