use std::sync::Arc;

use axum::Router;
use helper::user::create_user_and_get_token;
use hyper::StatusCode;
use tower::ServiceExt;

use trailsbuddy_backend_rust::{
    app::build_app_routes, database::AppDatabase,
    handlers::wallet::get_bal::Response as GetBalResponse, utils::get_epoch_ts,
};

use crate::helper::{build_get_request, get_database, GenericResponse};

mod helper;

const GET_BAL_PATH: &str = "/api/v1/wallet/getBalance";

async fn get_bal_unauthorized(app: Router) {
    let request = build_get_request(GET_BAL_PATH, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(response.success, false);
    assert_eq!(response.message, "Missing token".to_owned());
}

async fn get_bal_new_user(app: Router, db: Arc<AppDatabase>) {
    let ts = get_epoch_ts();
    let phone = format!("{}", ts);
    let name = "get_bal_new_user";
    let res = create_user_and_get_token(app.clone(), db, &phone, name, false).await;
    let request = build_get_request(GET_BAL_PATH, Some(&res.token));
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GetBalResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(response.success, true);
    assert_eq!(response.balance.real(), 0);
    assert_eq!(response.balance.bonus(), 0);
    assert_eq!(response.balance.withdrawable(), 0);
}

#[tokio::test]
async fn test_wallet_balance() {
    let db_client = get_database().await;
    let db_client = Arc::new(db_client);
    let app = build_app_routes(db_client.clone());
    tokio::join!(
        get_bal_unauthorized(app.clone()),
        get_bal_new_user(app.clone(), db_client.clone()),
    );
}
