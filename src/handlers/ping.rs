use axum::Json;

use crate::models::GenericResponse;

/// Ping endpoint
///
/// Ping the server to get a static response
#[utoipa::path(
    get,
    path = "/api/v1/ping",
    responses(
        (status = 200, description = "Get success response from server", body=GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn ping_handler() -> Json<GenericResponse> {
    let res = GenericResponse {
        success: true,
        message: "Server running successfully!".to_owned(),
    };
    Json(res)
}
