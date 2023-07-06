use std::{collections::HashMap, sync::Arc};

use axum::{body::Body, http::Request, routing::MethodRouter, Router};
use dotenvy::dotenv;
use trailsbuddy_backend_rust::database::AppDatabase;

pub fn req_body(path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

pub fn req(path: &str, token: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(path);
    let builder = if let Some(token) = token {
        builder.header("Authorization", format!("Bearer {token}"))
    } else {
        builder
    };
    builder.body(Body::empty()).unwrap()
}

pub async fn get_app(path: &str, method_router: MethodRouter<Arc<AppDatabase>>) -> Router {
    // import .env file
    dotenv().ok();
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    let db_client = Arc::new(db_client);
    let app = Router::new();
    let app = app.route(path, method_router);
    let app = app.with_state(db_client);
    app
}

pub async fn create_app(all_routes: HashMap<&str, MethodRouter<Arc<AppDatabase>>>) -> Router {
    // import .env file
    dotenv().ok();
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    let db_client = Arc::new(db_client);
    let mut app = Router::new();
    for (k, v) in all_routes {
        app = app.route(k, v);
    }
    let app = app.with_state(db_client);
    app
}
