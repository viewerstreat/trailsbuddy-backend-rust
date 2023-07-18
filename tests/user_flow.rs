use std::sync::Arc;

use axum::{http::StatusCode, Router};
use tower::ServiceExt; // for `oneshot` and `ready`

use crate::helper::{
    build_post_request, create_user, create_user_and_get_token, create_user_with_body,
    get_database,
    helper::{generate_uniq_phone, USER_FLOW_PREFIX},
};
use trailsbuddy_backend_rust::{
    app::build_app_routes,
    database::AppDatabase,
    models::{user::LoginScheme, wallet::Money, GenericResponse},
    utils::get_epoch_ts,
};

mod helper;

const CREATE_USER_PATH: &str = "/api/v1/user/create";

async fn test_empty_body(app: Router) {
    let body = r#"{}"#;
    let request = build_post_request(CREATE_USER_PATH, body, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

async fn test_missing_phone(app: Router) {
    let body = r#"{"name": "", "email": "validemail@internet.com", "profilePic": "invalidurl"}"#;
    let request = build_post_request(CREATE_USER_PATH, body, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

async fn test_invalid_phone(app: Router) {
    let body = r#"{"name": "abcd", "phone": "1234"}"#;
    let request = build_post_request(CREATE_USER_PATH, body, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, false);
    assert_eq!(response.message.contains("Phone must be 10 digit"), true);
}

async fn test_invalid_char_in_phone(app: Router) {
    let body = r#"{"name": "abcd", "phone": "1234O12341"}"#;
    let request = build_post_request(CREATE_USER_PATH, body, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, false);
    assert_eq!(response.message.contains("Phone must be all digits"), true);
}

async fn test_duplicate_phone(app: Router, phone: &str) {
    let body = format!(
        "{{\"name\": \"test_duplicate_phone\", \"phone\": \"{}\"}}",
        phone
    );
    create_user(app.clone(), phone, "test_duplicate_phone").await;
    let request = build_post_request(CREATE_USER_PATH, &body, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, false);
    let msg = "User already exists with same phone";
    assert_eq!(response.message.contains(msg), true);
}

async fn test_duplicate_email(app: Router, phone1: &str, phone2: &str) {
    let email = format!("{}@email.com", get_epoch_ts());
    let body1 = format!(
        "{{\"name\": \"test_duplicate_email\", \"phone\": \"{}\", \"email\":\"{}\"}}",
        phone1, &email
    );
    let body2 = format!(
        "{{\"name\": \"test_duplicate_email\", \"phone\": \"{}\", \"email\":\"{}\"}}",
        phone2, &email
    );
    create_user_with_body(app.clone(), &body1).await;
    let request = build_post_request(CREATE_USER_PATH, &body2, None);
    let res = app.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let response: GenericResponse = serde_json::from_slice(&body).unwrap();
    println!("{:?}", response);
    assert_eq!(response.success, false);
    let msg = "User already exists with same email";
    assert_eq!(response.message.contains(msg), true);
}

async fn test_successful_signup(app: Router, db: Arc<AppDatabase>, phone: &str) {
    let name = "test_successful_signup";
    let res = create_user_and_get_token(app, db, phone, name, true).await;
    assert_eq!(res.success, true);
    assert_eq!(res.data.id.ge(&0), true);
    assert_eq!(res.data.name.as_str(), name);
    assert_eq!(res.data.phone.as_ref().unwrap(), phone);
    assert_eq!(res.data.email.is_none(), true);
    assert_eq!(res.data.profile_pic, None);
    assert_eq!(res.data.login_scheme, LoginScheme::OTP_BASED);
    assert_eq!(res.data.is_active, true);
    assert_eq!(res.data.has_used_referral_code, Some(false));
    assert_eq!(res.data.referral_code.is_some(), true);
    assert_eq!(res.data.referred_by, None);
    assert_eq!(res.data.total_played, Some(0));
    assert_eq!(res.data.contest_won, Some(0));
    assert_eq!(res.data.total_earning, Some(Money::default()));
    assert_eq!(res.data.fcm_tokens, None);
}

#[tokio::test]
async fn test_user_signup() {
    let phone1 = generate_uniq_phone(USER_FLOW_PREFIX);
    let phone2 = generate_uniq_phone(USER_FLOW_PREFIX);
    let phone3 = generate_uniq_phone(USER_FLOW_PREFIX);
    let phone4 = generate_uniq_phone(USER_FLOW_PREFIX);
    let db_client = get_database().await;
    let db_client = Arc::new(db_client);
    let app = build_app_routes(db_client.clone());
    tokio::join!(
        test_empty_body(app.clone()),
        test_missing_phone(app.clone()),
        test_invalid_phone(app.clone()),
        test_invalid_char_in_phone(app.clone()),
        test_duplicate_phone(app.clone(), &phone1),
        test_duplicate_email(app.clone(), &phone2, &phone3),
        test_successful_signup(app, db_client, &phone4),
    );
}
