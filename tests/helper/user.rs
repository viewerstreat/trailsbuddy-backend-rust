use std::sync::Arc;

use axum::{http::StatusCode, Router};
use mongodb::bson::doc;
use tower::ServiceExt;

use trailsbuddy_backend_rust::{
    constants::*,
    database::AppDatabase,
    handlers::user::check_otp::Response as CheckOtpResponse,
    models::{otp::Otp, user::User},
    utils::get_epoch_ts,
};

use crate::helper::helper::{build_get_request, build_post_request, GenericResponse};

pub async fn create_user(app: Router, phone: &str, name: &str) {
    let body = format!("{{\"name\": \"{}\", \"phone\": \"{}\"}}", name, phone);
    create_user_with_body(app, &body).await;
}

pub async fn create_user_with_body(app: Router, body: &str) {
    let request = build_post_request("/api/v1/user/create", &body);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, true);
    assert_eq!(response.message, "User created".to_owned());
}

pub async fn verify_user(app: Router, phone: &str) {
    let path = format!("/api/v1/user/verify?phone={}", phone);
    let request = build_get_request(path.as_str(), None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, true);
    assert_eq!(response.message, "Otp generated".to_owned());
}

pub async fn check_otp(app: Router, phone: &str, otp: &str) -> CheckOtpResponse {
    let path = format!("/api/v1/user/checkOtp?phone={}&otp={}", phone, otp);
    let request = build_get_request(path.as_str(), None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: CheckOtpResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, true);
    assert_eq!(response.data.phone.as_ref().unwrap(), phone);
    assert_eq!(response.token.is_empty(), false);
    assert_eq!(response.refresh_token.is_empty(), false);
    response
}

async fn get_user(db: Arc<AppDatabase>, phone: &str) -> Option<User> {
    let filter = doc! {"phone": phone};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await
        .unwrap();
    user
}

async fn get_otp_val(db: Arc<AppDatabase>, phone: &str) -> String {
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

pub async fn create_user_and_get_token(
    app: Router,
    db: Arc<AppDatabase>,
    phone: &str,
    name: &str,
    panic_if_exists: bool,
) -> CheckOtpResponse {
    let user = get_user(db.clone(), phone).await;
    if panic_if_exists && user.is_some() {
        panic!("User already exists with phone: {}", phone);
    }
    if user.is_some() {
        verify_user(app.clone(), phone).await;
    } else {
        create_user(app.clone(), phone, name).await;
    }
    let otp = get_otp_val(db, phone).await;
    check_otp(app, phone, &otp).await
}
