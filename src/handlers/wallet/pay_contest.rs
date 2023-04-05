use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::get_bal::get_user_balance;
use crate::{
    constants::*,
    handlers::play_tracker::get::{insert_new_play_tracker, validate_contest},
    jwt::JwtClaims,
    models::{
        play_tracker::{PlayTracker, PlayTrackerStatus},
        wallet::{Money, Wallet, WalletTransaction},
    },
    utils::{get_epoch_ts, parse_object_id, AppError},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqBody {
    contest_id: String,
    bonus_money_amount: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: PlayTracker,
}

pub async fn pay_contest_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Json(body): Json<ReqBody>,
) -> Result<Json<Response>, AppError> {
    let contest_id = parse_object_id(&body.contest_id, "Not able to parse contestId")?;
    let (contest_result, play_tracker_result) = tokio::join!(
        validate_contest(&db, &contest_id),
        check_play_tracker(&db, &body.contest_id, claims.id)
    );
    let contest = contest_result?;
    let mut play_tracker = play_tracker_result?;
    let bonus_money_amount = body.bonus_money_amount.unwrap_or_default();
    debug_assert!(contest.entry_fee_max_bonus_money <= contest.entry_fee);
    if bonus_money_amount > contest.entry_fee_max_bonus_money {
        let err = format!(
            "entryFeeMaxBonusMoney is : {}",
            contest.entry_fee_max_bonus_money
        );
        let err = AppError::BadRequestErr(err);
        return Err(err);
    }
    let real_money_amount = (contest.entry_fee - bonus_money_amount) as u64;
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
            let balance_after =
                update_wallet(db, session, user_id, real_money_amount, bonus_money_amount).await?;
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
            update_play_tracker(db, session, user_id, &contest_id, &transaction_id).await?;
            Ok(())
        }
        .boxed()
    })
    .await?;
    play_tracker.status = PlayTrackerStatus::PAID;
    let res = Response {
        success: true,
        data: play_tracker,
    };

    Ok(Json(res))
}

async fn update_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    real: u64,
    bonus: u64,
) -> anyhow::Result<Money> {
    let ts = get_epoch_ts() as i64;
    let bonus = bonus as i64;
    let real = real as i64;
    let filter = doc! {
        "userId": user_id,
        "balance.real": {"$gte": real},
        "balance.bonus": {"$gte": bonus}
    };
    let update = doc! {
        "$set": {"updatedTs": ts},
        "$inc": {"balance.real": real * -1, "balance.bonus": bonus * -1}
    };
    let options = FindOneAndUpdateOptions::builder()
        .return_document(Some(ReturnDocument::After))
        .build();
    let wallet = db
        .find_one_and_update_with_session::<Wallet>(
            session,
            DB_NAME,
            COLL_WALLETS,
            filter,
            update,
            Some(options),
        )
        .await?
        .ok_or(anyhow::anyhow!("not able to update wallet"))?;
    Ok(wallet.balance())
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
    let transaction_id = db
        .insert_one_with_session::<WalletTransaction>(
            session,
            DB_NAME,
            COLL_WALLET_TRANSACTIONS,
            &transaction,
            None,
        )
        .await?;
    Ok(transaction_id)
}

async fn update_play_tracker(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    contest_id: &str,
    transaction_id: &str,
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

pub async fn check_play_tracker(
    db: &Arc<AppDatabase>,
    contest_id: &str,
    user_id: u32,
) -> Result<PlayTracker, AppError> {
    let filter = doc! {
        "contestId": contest_id,
        "userId": user_id,
    };
    let play_tracker = db
        .find_one::<PlayTracker>(DB_NAME, COLL_PLAY_TRACKERS, Some(filter), None)
        .await?;
    let Some(play_tracker) = play_tracker else {
        let play_tracker = insert_new_play_tracker(user_id, contest_id, db).await?;
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
