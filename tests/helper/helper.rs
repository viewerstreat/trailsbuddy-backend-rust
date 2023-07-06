use axum::{body::Body, http::Request};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use trailsbuddy_backend_rust::database::AppDatabase;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

pub async fn get_database() -> AppDatabase {
    // import .env file
    dotenv().ok();
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    db_client
}

pub fn build_post_request(path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
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
