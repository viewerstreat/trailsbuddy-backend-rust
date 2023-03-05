use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn ping_handler() -> impl IntoResponse {
    Json(json!({"success": true, "message": "Server running successfully!"}))
}
