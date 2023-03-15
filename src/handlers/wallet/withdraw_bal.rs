use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::add_bal::{
    update_wallet_transaction, updated_failed_transaction, TRANSACTION_ID_PARSE_ERR,
};
use crate::{
    constants::*,
    handlers::wallet::{get_bal::get_user_balance, model::*},
    jwt::JwtClaims,
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawBalInitReq {
    #[validate(range(min = "WITHDRAW_BAL_MIN_AMOUNT"))]
    amount: u64,
    #[validate(email)]
    receiver_upi_id: String,
}

pub async fn withdraw_bal_init_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<WithdrawBalInitReq>,
) -> Result<Json<JsonValue>, AppError> {
    let transaction = validate_request(&db, claims.id, &body).await?;
    let transaction_id = db
        .insert_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, &transaction, None)
        .await?;
    let res = json!({"success": true, "transactionId": &transaction_id});
    Ok(Json(res))
}

async fn validate_request(
    db: &Arc<AppDatabase>,
    user_id: u32,
    body: &WithdrawBalInitReq,
) -> Result<WalletTransaction, AppError> {
    let filter = doc! {
        "userId": user_id,
        "transactionType": WalltetTransactionType::Withdraw.to_bson()?,
        "status": WalletTransactionStatus::Pending.to_bson()?
    };
    let (transaction_result, balance_result) = tokio::join!(
        db.find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None),
        get_user_balance(db, user_id)
    );
    if transaction_result?.is_some() {
        let err = "Already a pending withdraw request exists";
        let err = AppError::BadRequestErr(err.into());
        tracing::debug!("{:?}", err);
        tracing::debug!("{:?}", body);
        return Err(err);
    }
    let user_balance = balance_result?.unwrap_or_default();
    if user_balance.real() < body.amount {
        let err = "Insufficient balance";
        let err = AppError::BadRequestErr(err.into());
        tracing::debug!("{:?}", err);
        tracing::debug!("{:?}", body);
        return Err(err);
    }
    let amount = Money::new(body.amount, 0);
    let transaction = WalletTransaction::withdraw_bal_init_trans(
        user_id,
        amount,
        user_balance,
        &body.receiver_upi_id,
    );

    Ok(transaction)
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawBalEndReq {
    #[validate(range(min = "WITHDRAW_BAL_MIN_AMOUNT"))]
    amount: u64,
    transaction_id: String,
    is_successful: bool,
    error_reason: Option<String>,
    tracking_id: Option<String>,
}

pub async fn withdraw_bal_end_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<WithdrawBalEndReq>,
) -> Result<Json<JsonValue>, AppError> {
    validate_end_request(&db, &body, claims.id).await?;
    if body.is_successful {
        handle_success_transaction(&db, &body, claims.id).await?;
    } else {
        handle_failed_transaction(&db, &body, claims.id).await?;
    }

    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

async fn handle_success_transaction(
    db: &Arc<AppDatabase>,
    body: &WithdrawBalEndReq,
    user_id: u32,
) -> Result<(), AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    db.execute_transaction(None, None, |db, session| {
        let tracking_id = body.tracking_id.clone();
        let amount = body.amount as i64;
        async move {
            let wallet = update_wallet(db, session, user_id, amount).await?;
            update_wallet_transaction(
                db,
                session,
                &transaction_id,
                &wallet.balance(),
                &tracking_id,
            )
            .await?;
            Ok(())
        }
        .boxed()
    })
    .await?;

    Ok(())
}

async fn update_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    amount: i64,
) -> anyhow::Result<Wallet> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user_id, "balance.real": {"$gte": amount}};
    let update = doc! {"$set": {"updatedTs": ts}, "$inc": {"balance.real": amount * -1}};
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
    Ok(wallet)
}

async fn handle_failed_transaction(
    db: &Arc<AppDatabase>,
    body: &WithdrawBalEndReq,
    user_id: u32,
) -> Result<(), AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    updated_failed_transaction(
        db,
        user_id,
        &transaction_id,
        &body.error_reason,
        &body.tracking_id,
    )
    .await?;
    Ok(())
}

async fn validate_end_request(
    db: &Arc<AppDatabase>,
    body: &WithdrawBalEndReq,
    user_id: u32,
) -> Result<(), AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    let filter = doc! {
        "_id": transaction_id,
        "userId": user_id,
        "status": WalletTransactionStatus::Pending.to_bson()?,
        "transactionType": WalltetTransactionType::Withdraw.to_bson()?
    };
    let (transaction_result, balance_result) = tokio::join!(
        db.find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None),
        get_user_balance(db, user_id)
    );
    let transaction =
        transaction_result?.ok_or(AppError::NotFound("transaction not found".into()))?;
    let user_balance = balance_result?.unwrap_or_default();
    let amount = Money::new(body.amount, 0);
    if transaction.amount() != amount {
        let err = AppError::BadRequestErr("amount do not match".into());
        return Err(err);
    }
    if user_balance.real() < amount.real() {
        let msg = format!(
            "Insufficient balance. Balance {}. Amount {}",
            user_balance, amount
        );
        let msg = Some(msg);
        updated_failed_transaction(db, user_id, &transaction_id, &msg, &body.tracking_id).await?;
        let err = AppError::BadRequestErr(msg.unwrap());
        return Err(err);
    }

    Ok(())
}
