use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::models::GenericResponse;

#[derive(Debug)]
pub enum AppError {
    BadRequestErr(String),
    NotFound(String),
    Auth(String),
    AnyError(anyhow::Error),
}

impl AppError {
    pub fn unknown_error() -> Self {
        Self::AnyError(anyhow::anyhow!("Unknown error"))
    }
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
                let response = GenericResponse {
                    success: false,
                    message: msg.to_owned(),
                };
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            Self::NotFound(msg) => {
                tracing::debug!("Not Found: {}", msg);
                let response = GenericResponse {
                    success: false,
                    message: msg.to_owned(),
                };
                (StatusCode::NOT_FOUND, Json(response)).into_response()
            }
            Self::Auth(msg) => {
                tracing::debug!("Unauthorized: {}", msg);
                let response = GenericResponse {
                    success: false,
                    message: msg.to_owned(),
                };
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
            Self::AnyError(err) => {
                let msg = format!("Something went wrong: {err}");
                tracing::debug!("{msg}");
                let response = GenericResponse {
                    success: false,
                    message: msg.to_owned(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
    }
}
