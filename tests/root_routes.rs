use axum::http::StatusCode;
use axum::{body::Body, http::Request, routing::get, Router};
use serde::Deserialize;
use tower::ServiceExt; // for `oneshot` and `ready`

use trailsbuddy_backend_rust::{
    handlers::{
        default::default_route_handler, global_404::global_404_handler, ping::ping_handler,
    },
    models::GenericResponse,
};

#[tokio::test]
async fn test_default_route_handler() {
    let app = Router::new().route("/", get(default_route_handler));
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let default_res: GenericResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(default_res.success, true);
    assert_eq!(default_res.message, "Server is running".to_owned());
}

#[tokio::test]
async fn test_global_404_handler() {
    let app = Router::new()
        .route("/", get(|| async {}))
        .fallback(global_404_handler);
    let req = Request::builder()
        .uri("/a-not-exiting-path")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    #[derive(Deserialize)]
    struct Response404 {
        success: bool,
    }
    let res_404: Response404 = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_404.success, false);
}

#[tokio::test]
async fn test_ping_handler() {
    let app = Router::new().route("/ping", get(ping_handler));
    let req = Request::builder()
        .uri("/ping")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    #[derive(Debug, Deserialize)]
    struct PingResponse {
        success: bool,
        message: String,
    }
    let res: PingResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.success, true);
    assert_eq!(res.message.as_str(), "Server running successfully!");
}
