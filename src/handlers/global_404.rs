use axum::http::Uri;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub async fn global_404_handler(uri: Uri) -> impl IntoResponse {
    let msg = format!("Route `{}` does not exist", uri);
    tracing::debug!(msg);
    let json_val = json!({"success": false, "message": msg});
    (StatusCode::NOT_FOUND, Json(json_val))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::get, Router};
    use serde::Deserialize;
    use tower::ServiceExt; // for `oneshot` and `ready`

    use super::*;

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
}
