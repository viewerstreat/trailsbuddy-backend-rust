use std::collections::HashMap;

use axum::{
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use mongodb::bson::doc;
use tower::ServiceExt;
use trailsbuddy_backend_rust::{
    constants::*,
    database::AppDatabase,
    handlers::{
        user::{
            check_otp::{check_otp_handler, Response as CheckOtpResponse},
            create::create_user_handler,
        },
        wallet::get_bal::{get_bal_handler, Response as GetBalResponse},
    },
    models::{otp::Otp, user::User},
    utils::get_epoch_ts,
};

use crate::helper::{create_app, req, req_body};

mod helper;

const CREATE_USER_PATH: &str = "/create-user";
const CHECK_OTP_PATH: &str = "/checkOtp";
const GET_BAL_PATH: &str = "/get-bal";

async fn get_app() -> Router {
    let create_user_method_router = post(create_user_handler);
    let get_bal_method_router = get(get_bal_handler);
    let check_otp_method_router = get(check_otp_handler);
    let mut all_routes = HashMap::new();
    all_routes.insert(CREATE_USER_PATH, create_user_method_router);
    all_routes.insert(GET_BAL_PATH, get_bal_method_router);
    all_routes.insert(CHECK_OTP_PATH, check_otp_method_router);
    let app = create_app(all_routes).await;
    app
}

async fn create_user(app: Router, phone: &str) {
    let body = format!("{{\"name\": \"abcd\", \"phone\": \"{}\"}}", phone);
    let res = app
        .oneshot(req_body(CREATE_USER_PATH, &body))
        .await
        .unwrap();
    println!("{:?}", res);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    #[derive(Debug, serde::Deserialize)]
    struct Response {
        success: bool,
        message: String,
    }
    let response: Response = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    // assert_eq!(res.status(), StatusCode::CREATED);
    assert_eq!(1, 0);
}

async fn get_otp_val(phone: &str) -> String {
    let db = AppDatabase::new().await.unwrap();
    let filter = doc! {"phone": &phone};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await
        .unwrap();
    let user = user.unwrap();
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"userId": user.id, "isUsed": false, "validTill": {"$gt": ts}};
    let otp = db
        .find_one::<Otp>(DB_NAME, COLL_OTP, Some(filter), None)
        .await
        .unwrap();
    let otp = otp.unwrap();
    otp.otp
}

async fn get_token(app: Router, phone: &str, otp: &str) -> String {
    let path = format!("{}?phone={}&otp={}", CHECK_OTP_PATH, phone, otp);
    let res = app.oneshot(req(&path, None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: CheckOtpResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(response.success, true);
    assert_eq!(response.token.is_empty(), false);
    assert_eq!(response.refresh_token.is_empty(), false);
    response.token
}

async fn get_bal_unauthorized(app: Router) {
    let res = app.oneshot(req(GET_BAL_PATH, None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

async fn get_bal_with_token(app: Router, phone: &str) {
    let otp = get_otp_val(phone).await;
    let token = get_token(app.clone(), phone, &otp).await;
    let res = app.oneshot(req(GET_BAL_PATH, Some(&token))).await.unwrap();
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
    let app = get_app().await;
    let ts = get_epoch_ts();
    let phone = format!("{}", ts);
    create_user(app.clone(), &phone).await;
    get_bal_unauthorized(app.clone()).await;
    get_bal_with_token(app.clone(), &phone).await;
}
