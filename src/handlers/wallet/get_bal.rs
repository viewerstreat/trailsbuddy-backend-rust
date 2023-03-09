use axum::{extract::State, Json};
use mockall_double::double;
use mongodb::bson::doc;
use serde::Serialize;
use std::sync::Arc;

use super::model::{Money, Wallet};
use crate::{constants::*, jwt::JwtClaims, utils::AppError};

#[double]
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

impl Default for Response {
    fn default() -> Self {
        let balance = Money::new(0, 0);
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
    let filter = doc! {"userId": claims.id};
    let wallet = db
        .find_one::<Wallet>(DB_NAME, COLL_WALLETS, Some(filter), None)
        .await?;
    let res = if let Some(wallet) = wallet {
        Response::new(wallet.balance())
    } else {
        Response::default()
    };
    Ok(Json(res))
}
