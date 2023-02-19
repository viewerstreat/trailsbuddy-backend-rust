use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = format!("Something went wrong: {}", self.0);
        let json_val = json!({"success": false, "message": msg});
        let res = (StatusCode::INTERNAL_SERVER_ERROR, Json(json_val));
        res.into_response()
    }
}
