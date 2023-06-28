use axum::{extract::State, Json};
use mongodb::bson::doc;
use serde::Serialize;
use std::sync::Arc;

use crate::{database::AppDatabase, jwt::JwtClaims, models::wallet::Money, utils::AppError};

use super::helper::get_user_balance;

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
