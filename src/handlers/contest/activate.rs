use axum::{extract::State, Json};
use mongodb::bson::{doc, Document};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::contest::{Contest, ContestStatus, ContestWithQuestion},
    utils::{get_epoch_ts, parse_object_id, AppError},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    contest_id: String,
}

pub async fn activate_contest_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
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
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

pub async fn inactivate_contest_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
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
    let res = json!({"success": true, "message": "Updated successfully"});
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
