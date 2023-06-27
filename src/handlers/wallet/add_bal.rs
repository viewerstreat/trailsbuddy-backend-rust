use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::get_bal::get_user_balance;
use crate::{
    constants::*,
    jwt::JwtClaims,
    models::wallet::{
        Money, Wallet, WalletTransaction, WalletTransactionStatus, WalltetTransactionType,
    },
    utils::{get_epoch_ts, parse_object_id, AppError, ValidatedBody},
};

use crate::database::AppDatabase;

#[derive(Debug, Deserialize, Validate)]
pub struct AddBalInitReq {
    #[validate(range(min = 1))]
    amount: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBalInitRes {
    success: bool,
    transaction_id: String,
    app_upi_id: String,
}

pub async fn add_bal_init_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<AddBalInitReq>,
) -> Result<Json<AddBalInitRes>, AppError> {
    let app_upi_id = std::env::var("APP_UPI_ID")?;
    let amount = Money::new(body.amount, 0);
    let balance_before = get_user_balance(&db, claims.id).await?.unwrap_or_default();
    let transaction = WalletTransaction::add_bal_init_trans(claims.id, amount, balance_before);
    let transaction_id = db
        .insert_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, &transaction, None)
        .await?;
    let res = AddBalInitRes {
        success: true,
        transaction_id,
        app_upi_id,
    };
    Ok(Json(res))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddBalEndReq {
    #[validate(range(min = 1))]
    amount: u64,
    transaction_id: String,
    is_successful: bool,
    error_reason: Option<String>,
    tracking_id: Option<String>,
}

pub const TRANSACTION_ID_PARSE_ERR: &str = "Not able to parse transactionId value";

pub async fn add_bal_end_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<AddBalEndReq>,
) -> Result<Json<JsonValue>, AppError> {
    validate_transaction(&claims, &db, &body).await?;
    if body.is_successful {
        handle_success_transaction(&claims, &db, &body).await
    } else {
        handle_failed_transaction(&claims, &db, &body).await
    }
}

async fn handle_success_transaction(
    claims: &JwtClaims,
    db: &Arc<AppDatabase>,
    body: &AddBalEndReq,
) -> Result<Json<JsonValue>, AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    let user_id = claims.id;
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
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

async fn update_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    amount: i64,
) -> anyhow::Result<Wallet> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user_id};
    let update = doc! {
        "$set": {"updatedTs": ts},
        "$inc": {"balance.real": amount, "balance.bonus": 0}
    };
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
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

pub async fn update_wallet_transaction(
    db: &AppDatabase,
    session: &mut ClientSession,
    transaction_id: &ObjectId,
    balance_after: &Money,
    tracking_id: &Option<String>,
) -> anyhow::Result<()> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"_id": transaction_id};
    let update = doc! {
        "$set": {
            "balanceAfter": balance_after.to_bson()?,
            "status": WalletTransactionStatus::Completed.to_bson()?,
            "trackingId": tracking_id,
            "updatedTs": ts
        }
    };
    db.update_one_with_session(
        session,
        DB_NAME,
        COLL_WALLET_TRANSACTIONS,
        filter,
        update,
        None,
    )
    .await?;
    Ok(())
}

async fn handle_failed_transaction(
    claims: &JwtClaims,
    db: &Arc<AppDatabase>,
    body: &AddBalEndReq,
) -> Result<Json<JsonValue>, AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    updated_failed_transaction(
        db,
        claims.id,
        &transaction_id,
        &body.error_reason,
        &body.tracking_id,
    )
    .await?;
    let res = json!({"success": true, "message": "Updated successfully"});
    Ok(Json(res))
}

async fn validate_transaction(
    claims: &JwtClaims,
    db: &Arc<AppDatabase>,
    body: &AddBalEndReq,
) -> Result<(), AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    let filter = doc! {
        "_id": transaction_id,
        "userId": claims.id,
        "status": WalletTransactionStatus::Pending.to_bson()?,
        "transactionType": WalltetTransactionType::AddBalance.to_bson()?
    };
    let (transaction_result, balance_result) = tokio::join!(
        db.find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None),
        get_user_balance(db, claims.id)
    );
    let transaction =
        transaction_result?.ok_or(AppError::NotFound("transaction not found".into()))?;
    let user_balance = balance_result?.unwrap_or_default();
    let amount = Money::new(body.amount, 0);
    if transaction.amount() != amount {
        let err = AppError::BadRequestErr("amount do not match".into());
        return Err(err);
    }
    if user_balance != transaction.balance_before() {
        let msg = format!(
            "user balance {} does not match with transaction balanceBefore {}",
            user_balance,
            transaction.balance_before()
        );
        let msg = Some(msg);
        updated_failed_transaction(db, claims.id, &transaction_id, &msg, &body.tracking_id).await?;
        let err = AppError::BadRequestErr(msg.unwrap());
        return Err(err);
    }

    Ok(())
}

pub async fn updated_failed_transaction(
    db: &Arc<AppDatabase>,
    user_id: u32,
    transaction_id: &ObjectId,
    error_reason: &Option<String>,
    tracking_id: &Option<String>,
) -> Result<(), AppError> {
    let filter = doc! {"_id": transaction_id};
    let update = doc! {
        "$set": {
            "status": WalletTransactionStatus::Error.to_bson()?,
            "errorReason": error_reason,
            "trackingId": tracking_id,
            "updatedBy": user_id,
            "updatedTs": get_epoch_ts() as i64
        }
    };
    db.update_one(DB_NAME, COLL_WALLET_TRANSACTIONS, filter, update, None)
        .await?;
    Ok(())
}
