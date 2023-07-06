use mongodb::bson::doc;

use axum::{http::StatusCode, routing::post};
use serde::Deserialize;
use tower::ServiceExt; // for `oneshot` and `ready`

use crate::helper::{get_app, req_body};
use trailsbuddy_backend_rust::{
    constants::*,
    database::AppDatabase,
    handlers::user::create::create_user_handler,
    models::{
        otp::Otp,
        user::{LoginScheme, User},
        wallet::Money,
    },
    utils::get_epoch_ts,
};

mod helper;

#[derive(Debug, Deserialize)]
struct Response {
    success: bool,
    message: String,
}

#[tokio::test]
async fn test_user_signup_validations() {
    let ts = get_epoch_ts();
    let phone1 = format!("{}", ts);
    let phone2 = format!("{}", ts + 1);
    let phone3 = format!("{}", ts + 2);
    let phone4 = format!("{}", ts + 3);
    let path = "/create";
    let method_router = post(create_user_handler);
    let app = get_app(path, method_router).await;
    {
        // empty object request body
        let app = app.clone();
        let body = r#"{}"#;
        let res = app.oneshot(req_body(path, body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
    {
        // missing `phone` field
        let app = app.clone();
        let body =
            r#"{"name": "", "email": "validemail@internet.com", "profilePic": "invalidurl"}"#;
        let res = app.oneshot(req_body(path, body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
    {
        // invalid `phone` field
        let app = app.clone();
        let body = r#"{"name": "abcd", "phone": "1234"}"#;
        let res = app.oneshot(req_body(path, body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&body).unwrap();
        println!("{:?}", response);
        assert_eq!(response.success, false);
        assert_eq!(response.message.contains("Phone must be 10 digit"), true);
    }
    {
        // `phone` has 10 digits but contain invalid char
        let app = app.clone();
        let body = r#"{"name": "abcd", "phone": "1234O12341"}"#;
        let res = app.oneshot(req_body(path, body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&body).unwrap();
        println!("{:?}", response);
        assert_eq!(response.success, false);
        assert_eq!(response.message.contains("Phone must be all digits"), true);
    }
    {
        // duplicate phone
        let body = format!("{{\"name\": \"abcd\", \"phone\": \"{}\"}}", phone1);
        let app1 = app.clone();
        let res = app1.oneshot(req_body(path, &body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let app2 = app.clone();
        let res = app2.oneshot(req_body(path, &body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&body).unwrap();
        println!("{:?}", response);
        assert_eq!(response.success, false);
        assert_eq!(
            response
                .message
                .contains("User already exists with same phone"),
            true
        );
    }
    {
        // duplicate email check
        let email = format!("{}@email.com", get_epoch_ts());
        let body1 = format!(
            "{{\"name\": \"abcd\", \"phone\": \"{}\", \"email\":\"{}\"}}",
            phone2, &email
        );
        let body2 = format!(
            "{{\"name\": \"abcd\", \"phone\": \"{}\", \"email\":\"{}\"}}",
            phone3, &email
        );
        let app1 = app.clone();
        let res = app1.oneshot(req_body(path, &body1)).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let app2 = app.clone();
        let res = app2.oneshot(req_body(path, &body2)).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let response: Response = serde_json::from_slice(&body).unwrap();
        println!("{:?}", response);
        assert_eq!(response.success, false);
        assert_eq!(
            response
                .message
                .contains("User already exists with same email"),
            true
        );
    }
    {
        // successful creation data validation
        let body = format!("{{\"name\": \"abcd\", \"phone\": \"{}\"}}", phone4.clone());
        let app = app.clone();
        let res = app.oneshot(req_body(path, &body)).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        check_user_data_in_database(phone4).await;
    }
}

async fn check_user_data_in_database(phone: String) {
    let db = AppDatabase::new().await.unwrap();
    let ts = get_epoch_ts();
    let filter = doc! {"phone": &phone};
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, Some(filter), None)
        .await
        .unwrap();
    let user = user.unwrap();
    assert_eq!(user.id.ge(&0), true);
    assert_eq!(user.name.as_str(), "abcd");
    assert_eq!(user.phone.unwrap().as_str(), &phone);
    assert_eq!(user.email, None);
    assert_eq!(user.profile_pic, None);
    assert_eq!(user.login_scheme, LoginScheme::OTP_BASED);
    assert_eq!(user.is_active, true);
    assert_eq!(user.has_used_referral_code, Some(false));
    assert_eq!(user.referral_code.is_some(), true);
    assert_eq!(user.referred_by, None);
    assert_eq!(user.total_played, Some(0));
    assert_eq!(user.contest_won, Some(0));
    assert_eq!(user.total_earning, Some(Money::default()));
    assert_eq!(user.fcm_tokens, None);
    let filter = doc! {"userId": user.id};
    let otp = db
        .find_one::<Otp>(DB_NAME, COLL_OTP, Some(filter), None)
        .await
        .unwrap();
    let otp = otp.unwrap();
    assert_eq!(otp.otp.len(), OTP_LENGTH as usize);
    assert_eq!(otp.is_used, false);
    assert_eq!(otp.valid_till.ge(&ts), true);
}
