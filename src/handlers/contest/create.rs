use axum::{extract::State, Json};
use mongodb::bson::{doc, Document};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::contest::{Contest, ContestCategory, ContestProps, PrizeSelection},
    utils::{parse_object_id, AppError, ValidatedBody},
};

pub async fn create_contest_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ContestProps>,
) -> Result<Json<JsonValue>, AppError> {
    validate_body(&db, &body).await?;
    let mut contest = Contest::new(&body, claims.id);
    let inserted_id = db
        .insert_one::<Contest>(DB_NAME, COLL_CONTESTS, &contest, None)
        .await?;
    contest._id = Some(inserted_id);
    let res = json!({"success": true, "data": &contest});
    Ok(Json(res))
}

async fn validate_body(db: &Arc<AppDatabase>, body: &ContestProps) -> Result<(), AppError> {
    let (duplicate_check, movie_id_check) = tokio::join!(
        check_duplicate_title(&db, &body.title),
        validate_movie_id(&db, &body)
    );
    duplicate_check?;
    movie_id_check?;
    validate_entry_fee(&body)?;
    validate_prize_selection(&body)?;
    validate_prize_value(&body)?;
    validate_start_end_time(&body)?;
    Ok(())
}

async fn check_duplicate_title(db: &Arc<AppDatabase>, title: &str) -> Result<(), AppError> {
    let filter = doc! {"title": title};
    let result = db
        .find_one::<Contest>(DB_NAME, COLL_CONTESTS, Some(filter), None)
        .await?;
    if result.is_some() {
        let err = AppError::BadRequestErr("Duplicate contest title".into());
        return Err(err);
    }
    Ok(())
}

async fn validate_movie_id(db: &Arc<AppDatabase>, body: &ContestProps) -> Result<(), AppError> {
    match body.category {
        ContestCategory::Others => {
            if body.movie_id.is_some() {
                let msg = "movieId should be blank for `others` category";
                let err = AppError::BadRequestErr(msg.into());
                return Err(err);
            }
        }
        ContestCategory::Movie => {
            let movie_id = body
                .movie_id
                .as_ref()
                .ok_or(AppError::BadRequestErr("movieId is required".into()))?;
            let msg = "not able to parse movieId";
            let oid = parse_object_id(&movie_id, msg)?;
            let filter = doc! {"_id": oid, "isActive": true};
            db.find_one::<Document>(DB_NAME, COLL_MOVIES, Some(filter), None)
                .await?
                .ok_or(AppError::BadRequestErr("movie not found".into()))?;
        }
    };
    Ok(())
}

fn validate_entry_fee(body: &ContestProps) -> Result<(), AppError> {
    if body.entry_fee_max_bonus_money > body.entry_fee {
        let msg = "entryFeeMaxBonusMoney should be less than entryFee";
        return Err(AppError::BadRequestErr(msg.into()));
    }
    Ok(())
}

fn validate_prize_selection(body: &ContestProps) -> Result<(), AppError> {
    match body.prize_selection {
        PrizeSelection::TOP_WINNERS => {
            body.top_winners_count
                .ok_or(AppError::BadRequestErr("topWinnersCount required".into()))?;
        }
        PrizeSelection::RATIO_BASED => {
            let (numerator, denominator) = body
                .prize_ratio_numerator
                .and_then(|numerator| {
                    body.prize_ratio_denominator
                        .and_then(|denominator| Some((numerator, denominator)))
                })
                .ok_or(AppError::BadRequestErr(
                    "prizeRatioNumerator & prizeRatioDenominator required".into(),
                ))?;
            if numerator > denominator {
                let msg = "prizeRatioNumerator must be less than prizeRatioDenominator";
                let err = AppError::BadRequestErr(msg.into());
                return Err(err);
            }
        }
    };

    Ok(())
}

fn validate_prize_value(body: &ContestProps) -> Result<(), AppError> {
    if body.prize_value_real_money == 0 && body.prize_value_bonus_money == 0 {
        let msg = "prizeValueRealMoney & prizeValueBonusMoney both cannot be zero";
        let err = AppError::BadRequestErr(msg.into());
        return Err(err);
    }

    Ok(())
}

fn validate_start_end_time(body: &ContestProps) -> Result<(), AppError> {
    if body.start_time >= body.end_time {
        let msg = "startTime should be less than endTime";
        let err = AppError::BadRequestErr(msg.into());
        return Err(err);
    }

    Ok(())
}
