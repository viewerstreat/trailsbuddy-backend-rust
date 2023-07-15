use axum::{extract::State, Json};
use std::sync::Arc;

use crate::{database::AppDatabase, jwt::JwtClaims, models::*, utils::AppError};

use super::helper::get_user_balance;

/// get user balance
#[utoipa::path(
    get,
    path = "/api/v1/wallet/getBalance",
    params(GetNotiReq, ("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "balance", body = GetBalResponse),
        (status = StatusCode::UNAUTHORIZED, description = "balance", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn get_bal_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<GetBalResponse>, AppError> {
    let balance = get_user_balance(&db, claims.id).await?.unwrap_or_default();
    let res = GetBalResponse::new(balance);
    Ok(Json(res))
}
