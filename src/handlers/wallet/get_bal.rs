use axum::{extract::State, Json};
use mongodb::bson::doc;
use serde::Serialize;
use std::sync::Arc;

use super::model::{Money, Wallet};
use crate::{constants::*, jwt::JwtClaims, utils::AppError};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    balance: Money,
}
impl Response {
    fn new(balance: Money) -> Self {
        Self {
            success: true,
            balance,
        }
    }
}

pub async fn get_bal_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<Response>, AppError> {
    let balance = get_user_balance(&db, claims.id).await?.unwrap_or_default();
    let res = Response::new(balance);
    Ok(Json(res))
}

pub async fn get_user_balance(
    db: &Arc<AppDatabase>,
    user_id: u32,
) -> anyhow::Result<Option<Money>> {
    let filter = doc! {"userId": user_id};
    let wallet = db
        .find_one::<Wallet>(DB_NAME, COLL_WALLETS, Some(filter), None)
        .await?;
    let balance = wallet.and_then(|wallet| Some(wallet.balance()));
    Ok(balance)
}
