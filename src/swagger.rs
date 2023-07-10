use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::ping::ping_handler,
        crate::handlers::default::default_route_handler,
        crate::handlers::temp_api::temp_api_get_token,
        crate::handlers::temp_api::temp_api_get_otp,
        crate::handlers::user::create::create_user_handler,
        crate::handlers::user::login::login_handler,
        crate::handlers::user::referral::use_referral_code_handler
    ),
    components(
        schemas(
            crate::models::GenericResponse,
            crate::handlers::user::create::CreateUserReq,
            crate::handlers::user::login::LoginRequest,
            crate::handlers::user::login::SocialLoginScheme,
            crate::handlers::user::login::Response,
            crate::handlers::user::referral::ReqBody,
            crate::models::user::User,
            crate::models::user::LoginScheme,
            crate::models::wallet::Money
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Debugging API", description = "API for debugging purposes"),
        (name = "App User API", description = "API for app user functionalities"),
        (name = "Admin API", description = "API for admin functionalities")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "authorization",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("authorization"))),
            )
        }
    }
}
