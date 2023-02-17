use axum::body::{boxed, Body};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, IntoMakeService};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::ServiceBuilderExt;

use crate::constants::REQUEST_TIMEOUT_SECS;

/// Initializes the app with all routes and middlewares
pub fn build() -> IntoMakeService<Router> {
    tracing::debug!("Initializing the app");
    // create the cors layer for middleware
    let cors_layer = CorsLayer::permissive();
    // create the timeout layer for middleware
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS));
    // combine all middlewares with ServiceBuilder
    let middleware = ServiceBuilder::new()
        .layer(timeout_layer)
        .map_response_body(boxed)
        .compression();
    // create the app instance with all routes and middleware
    let app: Router<(), Body> = Router::new()
        .route("/", get(default_route_handler))
        .layer(cors_layer)
        .layer(middleware);
    // return the IntoMakeService instance
    app.into_make_service()
}

#[derive(Debug, Serialize, Deserialize)]
struct DefaultResponse {
    success: bool,
    message: String,
}

/// Handler function for default route "/"
/// Returns a JSON response with 200 status code
async fn default_route_handler() -> impl IntoResponse {
    let response = DefaultResponse {
        success: true,
        message: "Server is running".to_string(),
    };
    (StatusCode::OK, Json(response))
}

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, TcpListener};

    use axum::body::Body;
    use axum::http::Request;

    use super::*;

    #[tokio::test]
    async fn test_app_default_route() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
        let listener = TcpListener::bind(&addr).unwrap();
        let app = build();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app)
                .await
                .unwrap();
        });

        let client = hyper::Client::new();

        let req_uri = format!("http://{}", addr);
        let response = client
            .request(Request::builder().uri(req_uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let default_res: DefaultResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(default_res.success, true);
        assert_eq!(default_res.message, "Server is running".to_owned());
    }
}
