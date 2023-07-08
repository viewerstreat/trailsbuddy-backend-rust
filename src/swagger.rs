use utoipa::OpenApi;

use crate::handlers::ping;
use crate::models::GenericResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        ping::ping_handler
    ),
    components(
                schemas(GenericResponse)
            ),
    tags(
        (name = "trailsbuddy-backend-rust", description = "Backend API for Trailsbuddy application")
    )
)]
pub struct ApiDoc;
