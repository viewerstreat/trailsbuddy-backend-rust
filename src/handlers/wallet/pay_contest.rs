use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    handlers::{
        play_tracker::get::{get_contest_id, insert_new_play_tracker, validate_contest},
        wallet::helper::{get_user_balance, update_wallet_with_session},
    },
    jwt::JwtClaims,
    models::*,
    utils::{get_epoch_ts, parse_object_id, AppError},
};

use super::helper::insert_wallet_transaction_session;

/// Pay contest
///
/// Pay contest and get PlayTracker
#[utoipa::path(
    post,
    path = "/api/v1/wallet/payContest",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = PayContestReqBody,
    responses(
        (status = StatusCode::OK, description = "Pay contest successful", body = PlayTrackerResponse),
    ),
    tag = "App User API"
)]
pub async fn pay_contest_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<PayContestReqBody>,
) -> Result<Json<PlayTrackerResponse>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let contest = validate_contest(&db, &contest_id).await?;
    let mut play_tracker = check_play_tracker(&db, &contest, claims.id).await?;
    let bonus_money_amount = body.bonus_money_amount.unwrap_or_default();
    debug_assert!(contest.props.entry_fee_max_bonus_money <= contest.props.entry_fee);
    if bonus_money_amount > contest.props.entry_fee_max_bonus_money {
        let err = format!(
            "entryFeeMaxBonusMoney is : {}",
            contest.props.entry_fee_max_bonus_money
        );
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }
    let real_money_amount = (contest.props.entry_fee - bonus_money_amount) as u64;
    let bonus_money_amount = bonus_money_amount as u64;
    let user_balance = get_user_balance(&db, claims.id).await?.unwrap_or_default();
    if user_balance.real() < real_money_amount {
        let err = format!(
            "Insufficient user balance, real money requuired: {}",
            real_money_amount
        );
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }
    if user_balance.bonus() < bonus_money_amount {
        let err = format!(
            "Insufficient user balance, bonusmoney requuired: {}",
            bonus_money_amount
        );
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }
    db.execute_transaction(None, None, |db, session| {
        let user_id = claims.id;
        let contest_id = body.contest_id.clone();
        async move {
            let (_, balance_after) = update_wallet_with_session(
                db,
                session,
                user_id,
                real_money_amount,
                bonus_money_amount,
                true,
                false,
            )
            .await?;
            let transaction_id = update_wallet_transaction(
                db,
                session,
                user_id,
                &contest_id,
                real_money_amount,
                bonus_money_amount,
                user_balance,
                balance_after,
            )
            .await?;
            let paid_amount = Money::new(real_money_amount, bonus_money_amount);
            update_play_tracker(
                db,
                session,
                user_id,
                &contest_id,
                &transaction_id,
                paid_amount,
            )
            .await?;
            Ok(())
        }
        .boxed()
    })
    .await?;
    play_tracker.status = PlayTrackerStatus::PAID;
    play_tracker.answers = None;
    let res = PlayTrackerResponse {
        success: true,
        data: play_tracker,
    };

    Ok(Json(res))
}

async fn update_wallet_transaction(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    contest_id: &str,
    real: u64,
    bonus: u64,
    balance_before: Money,
    balance_after: Money,
) -> anyhow::Result<String> {
    let transaction = WalletTransaction::pay_for_contest_trans(
        user_id,
        contest_id,
        real,
        bonus,
        balance_before,
        balance_after,
    );
    let transaction_id = insert_wallet_transaction_session(db, session, &transaction).await?;
    Ok(transaction_id)
}

async fn update_play_tracker(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    contest_id: &str,
    transaction_id: &str,
    paid_amount: Money,
) -> anyhow::Result<PlayTracker> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
    };
    let update = doc! {
        "$set": {
            "walletTransactionId": transaction_id,
            "status": PlayTrackerStatus::PAID.to_bson()?,
            "paidAmount": paid_amount.to_bson()?,
            "paidTs": ts,
            "updatedTs": ts,
            "updatedBy": user_id
        }
    };
    let options = FindOneAndUpdateOptions::builder()
        .return_document(Some(ReturnDocument::After))
        .build();
    let play_tracker = db
        .find_one_and_update_with_session::<PlayTracker>(
            session,
            DB_NAME,
            COLL_PLAY_TRACKERS,
            filter,
            update,
            Some(options),
        )
        .await?
        .ok_or(anyhow::anyhow!("not able to update wallet"))?;
    Ok(play_tracker)
}

async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest: &ContestWithQuestion,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let contest_id = get_contest_id(contest)?;
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
    };
    let play_tracker = db
        .find_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?;
    let Some(play_tracker) = play_tracker else {
        let play_tracker = insert_new_play_tracker(user_id, contest, db).await?;
        return Ok(play_tracker);
    };
    if play_tracker.status == PlayTrackerStatus::PAID {
        let err = "contest already paid for the user";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    if play_tracker.status == PlayTrackerStatus::STARTED {
        let err = "contest already started for the user";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    if play_tracker.status == PlayTrackerStatus::FINISHED {
        let err = "contest already finished for the user";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    Ok(play_tracker)
}
