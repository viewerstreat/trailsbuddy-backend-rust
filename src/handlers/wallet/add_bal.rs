use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    database::AppDatabase,
    handlers::wallet::helper::{get_user_balance, get_wallet_transaction},
    jwt::JwtClaims,
    models::*,
    utils::{parse_object_id, AppError, ValidatedBody},
};

use super::helper::{
    insert_wallet_transaction, update_wallet_transaction_session, update_wallet_with_session,
    updated_failed_transaction,
};

/// Add balance initialize
///
/// Initialize add balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/addBalanceInit",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = AddBalInitReq,
    responses(
        (status = StatusCode::OK, description = "Add balance initialized", body = AddBalInitRes),
    ),
    tag = "App User API"
)]
pub async fn add_bal_init_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<AddBalInitReq>,
) -> Result<Json<AddBalInitRes>, AppError> {
    let app_upi_id = std::env::var("APP_UPI_ID")?;
    let amount = Money::new(body.amount, 0);
    let balance_before = get_user_balance(&db, claims.id).await?.unwrap_or_default();
    let transaction = WalletTransaction::add_bal_init_trans(claims.id, amount, balance_before);
    let transaction_id = insert_wallet_transaction(&db, &transaction).await?;
    let res = AddBalInitRes {
        success: true,
        transaction_id,
        app_upi_id,
    };
    Ok(Json(res))
}

pub const TRANSACTION_ID_PARSE_ERR: &str = "Not able to parse transactionId value";

/// Add balance finalize
///
/// Finalize add balance transaction
#[utoipa::path(
    post,
    path = "/api/v1/wallet/addBalanceEnd",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = AddBalEndReq,
    responses(
        (status = StatusCode::OK, description = "Add balance successful", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn add_bal_end_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<AddBalEndReq>,
) -> Result<Json<GenericResponse>, AppError> {
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
) -> Result<Json<GenericResponse>, AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    let user_id = claims.id;
    db.execute_transaction(None, None, |db, session| {
        let tracking_id = body.tracking_id.clone();
        let amount = body.amount;
        async move {
            let (_, balance_after) =
                update_wallet_with_session(db, session, user_id, amount, 0, false, false).await?;
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
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
    Ok(Json(res))
}

async fn handle_failed_transaction(
    claims: &JwtClaims,
    db: &Arc<AppDatabase>,
    body: &AddBalEndReq,
) -> Result<Json<GenericResponse>, AppError> {
    let transaction_id = parse_object_id(&body.transaction_id, TRANSACTION_ID_PARSE_ERR)?;
    updated_failed_transaction(
        db,
        claims.id,
        &transaction_id,
        &body.error_reason,
        &body.tracking_id,
    )
    .await?;
    let res = GenericResponse {
        success: true,
        message: "Updated successfully".to_owned(),
    };
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
    let filter = Some(filter);
    let (transaction_result, balance_result) = tokio::join!(
        get_wallet_transaction(db, filter),
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
