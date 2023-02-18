use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultResponse {
    pub success: bool,
    pub message: String,
}

/// Handler function for default route "/"
/// Returns a JSON response with 200 status code
pub async fn default_route_handler() -> impl IntoResponse {
    let response = DefaultResponse {
        success: true,
        message: "Server is running".to_string(),
    };
    (StatusCode::OK, Json(response))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt; // for `oneshot` and `ready`

    use super::*;

    #[tokio::test]
    async fn test_default_route_handler() {
        let app = Router::new().route("/", get(default_route_handler));
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let default_res: DefaultResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(default_res.success, true);
        assert_eq!(default_res.message, "Server is running".to_owned());
    }
}
