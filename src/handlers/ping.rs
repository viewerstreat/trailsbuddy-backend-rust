use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn ping_handler() -> impl IntoResponse {
    Json(json!({"success": true, "message": "Server running successfully!"}))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request, routing::get, Router};
    use serde::Deserialize;
    use tower::ServiceExt; // for `oneshot` and `ready`

    use super::*;

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
}
