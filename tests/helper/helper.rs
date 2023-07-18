use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dotenvy::dotenv;
use serde::de::DeserializeOwned;
use tower::ServiceExt;
use trailsbuddy_backend_rust::{
    database::AppDatabase,
    utils::{get_epoch_ts, get_random_num},
};

pub async fn get_database() -> AppDatabase {
    // import .env file
    dotenv().ok();
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    db_client
}

pub fn build_post_request(path: &str, body: &str, token: Option<&str>) -> Request<Body> {
    let builder = Request::builder()
        .uri(path)
        .method("POST")
        .header("Content-Type", "application/json");
    let builder = if let Some(token) = token {
        builder.header("Authorization", format!("Bearer {token}"))
    } else {
        builder
    };
    builder.body(Body::from(body.to_owned())).unwrap()
}

pub fn build_get_request(path: &str, token: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(path);
    let builder = if let Some(token) = token {
        builder.header("Authorization", format!("Bearer {token}"))
    } else {
        builder
    };
    builder.body(Body::empty()).unwrap()
}

pub async fn oneshot_request<T>(
    app: Router,
    body: Request<Body>,
    expected_status: Option<StatusCode>,
) -> T
where
    T: DeserializeOwned,
{
    let uri = body.uri().to_string();
    let res = app.oneshot(body).await.unwrap();
    let status = res.status();
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    println!("response for {} -> {}", uri, body);
    if let Some(expected_status) = expected_status {
        assert_eq!(status, expected_status);
    }
    let body: T = serde_json::from_str(body).unwrap();
    body
}

pub const USER_FLOW_PREFIX: u32 = 1;
pub const WALLET_BALANCE_PREFIX: u32 = 2;

pub fn generate_uniq_phone(prefix: u32) -> String {
    let ts = get_epoch_ts();
    let ts = ts % 1000000;
    let num = get_random_num(100u32, 1000);
    format!("{}{}{}", prefix, ts, num)
}
