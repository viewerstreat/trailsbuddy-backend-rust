use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequestErr(String),
    AnyError(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self::AnyError(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequestErr(msg) => {
                tracing::debug!("Bad request: {}", msg);
                let json_val = json!({"success": false, "message": msg});
                let res = (StatusCode::BAD_REQUEST, Json(json_val));
                res.into_response()
            }
            Self::AnyError(err) => {
                let msg = format!("Something went wrong: {err}");
                tracing::debug!("{msg}");
                let json_val = json!({"success": false, "message": msg});
                let res = (StatusCode::INTERNAL_SERVER_ERROR, Json(json_val));
                res.into_response()
            }
        }
    }
}
