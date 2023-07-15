use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::bson::doc;
use std::sync::Arc;

use super::{
    add_bal::TRANSACTION_ID_PARSE_ERR,
    helper::{
        insert_wallet_transaction, update_wallet_transaction_session, update_wallet_with_session,
        updated_failed_transaction,
    },
};
use crate::{
    database::AppDatabase,
    handlers::wallet::helper::{get_user_balance, get_wallet_transaction},
    jwt::JwtClaims,
    models::*,
    utils::{parse_object_id, AppError, ValidatedBody},
};

/// Withdraw balance initialize
///
/// Initialize withdraw balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/withdrawBalInit",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = WithdrawBalInitReq,
    responses(
        (status = StatusCode::OK, description = "Withdraw balance initialized", body = WithdrawInitRes),
    ),
    tag = "App User API"
)]
pub async fn withdraw_bal_init_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<WithdrawBalInitReq>,
) -> Result<Json<WithdrawInitRes>, AppError> {
    let transaction = validate_request(&db, claims.id, &body).await?;
    let transaction_id = insert_wallet_transaction(&db, &transaction).await?;
    let res = WithdrawInitRes {
        success: true,
        transaction_id,
    };
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
    let filter = Some(filter);
    let (transaction_result, balance_result) = tokio::join!(
        get_wallet_transaction(db, filter),
        get_user_balance(db, user_id)
    );
    // If there is already a pending withdraw request then disallow to create another one
    // In all possibilities it must be some kind of errorneous scenario which should be investigated
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
    if user_balance.withdrawable() < body.amount {
        let err = "Not enough withdrawable balance";
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

/// Withdraw balance finalize
///
/// Finalize withdraw balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/withdrawBalanceEnd",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = WithdrawBalEndReq,
    responses(
        (status = StatusCode::OK, description = "Withdraw balance finalized", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn withdraw_bal_end_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<WithdrawBalEndReq>,
) -> Result<Json<GenericResponse>, AppError> {
    validate_end_request(&db, &body, claims.id).await?;
    if body.is_successful {
        handle_success_transaction(&db, &body, claims.id).await?;
    } else {
        handle_failed_transaction(&db, &body, claims.id).await?;
    }
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
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
        let amount = body.amount;
        async move {
            let (_, balance_after) =
                update_wallet_with_session(db, session, user_id, amount, 0, true, true).await?;
            update_wallet_transaction_session(
                db,
                session,
                &transaction_id,
                balance_after,
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
    let filter = Some(filter);
    let (transaction_result, balance_result) = tokio::join!(
        get_wallet_transaction(db, filter),
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
    if user_balance.withdrawable() < amount.real() {
        let err = format!(
            "Not enough withdrawable balance, available: {}",
            user_balance.withdrawable()
        );
        let err = AppError::BadRequestErr(err.into());
        tracing::debug!("{:?}", err);
        tracing::debug!("{:?}", body);
        return Err(err);
    }

    Ok(())
}
