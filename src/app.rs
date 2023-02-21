use axum::{
    body::{boxed, Body},
    http::{header, HeaderValue},
    routing::{get, post, IntoMakeService},
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
    ServiceBuilderExt,
};

use crate::database::get_db;
use crate::handlers::{
    clips::get_clips_handler, default::default_route_handler, global_404::global_404_handler,
};
use crate::{constants::REQUEST_TIMEOUT_SECS, handlers::user::create_user_handler};

/// Initializes the app with all routes and middlewares
pub async fn build() -> IntoMakeService<Router> {
    tracing::debug!("Initializing the app");
    // create a response header layer for middleware
    let server_header_value = HeaderValue::from_static("trailsbuddy-backend-rust");
    let set_response_header_layer =
        SetResponseHeaderLayer::if_not_present(header::SERVER, server_header_value);
    // create the trace layer for middleware
    // let trace_layer = TraceLayer::new_for_http();
    // create the cors layer for middleware
    let cors_layer = CorsLayer::permissive();
    // create the timeout layer for middleware
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS));
    // combine all middlewares with ServiceBuilder
    let middleware = ServiceBuilder::new()
        .layer(timeout_layer)
        .layer(cors_layer)
        .layer(set_response_header_layer)
        .map_response_body(boxed)
        .layer(TraceLayer::new_for_http())
        .compression()
        .into_inner();
    // create database client
    let db_client = get_db().await.expect("Unable to accquire database client");
    // create the app instance with all routes and middleware
    let app: Router<(), Body> = Router::new()
        .route("/", get(default_route_handler))
        .route("/clip", get(get_clips_handler))
        .route("/user", post(create_user_handler))
        .layer(middleware)
        .fallback(global_404_handler)
        .with_state(db_client);
    // return the IntoMakeService instance
    app.into_make_service()
}

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, TcpListener};

    use axum::body::Body;
    use axum::http::Request;

    use crate::handlers::default::DefaultResponse;

    use super::*;

    #[tokio::test]
    async fn test_app_default_route() {
        dotenvy::dotenv().ok();
        let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
        let listener = TcpListener::bind(&addr).unwrap();
        let app = build().await;

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
