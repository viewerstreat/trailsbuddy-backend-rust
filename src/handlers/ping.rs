use axum::Json;

use crate::models::GenericResponse;

/// Ping endpoint
///
/// Ping the server to get a static response
#[utoipa::path(
    get,
    path = "/api/v1/ping",
    responses(
        (status = 200, description = "Get success response from server", body=GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn ping_handler() -> Json<GenericResponse> {
    let res = GenericResponse {
        success: true,
        message: "Server running successfully!".to_owned(),
    };
    Json(res)
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
