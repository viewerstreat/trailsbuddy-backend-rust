use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, IntoMakeService};
use axum::{Json, Router};
use serde::Serialize;

pub fn build() -> IntoMakeService<Router> {
    tracing::debug!("Initializing the app");
    let app: Router<(), Body> = Router::new().route("/", get(default_route_handler));
    app.into_make_service()
}

#[derive(Serialize)]
struct DefaultResponse {
    success: bool,
    message: String,
}

async fn default_route_handler() -> impl IntoResponse {
    let response = DefaultResponse {
        success: true,
        message: "Server is running".to_string(),
    };
    (StatusCode::OK, Json(response))
}
