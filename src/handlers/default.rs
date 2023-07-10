use axum::Json;

use crate::models::GenericResponse;

/// Default endpoint
///
/// Returns a JSON response with 200 status code
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Get success response from server", body=GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn default_route_handler() -> Json<GenericResponse> {
    let response = GenericResponse {
        success: true,
        message: "Server is running".to_string(),
    };
    Json(response)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    use super::*;

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
}
