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
        crate::handlers::user::verify::verify_user_handler,
        crate::handlers::user::check_otp::check_otp_handler,
        crate::handlers::user::get_leaderboard::get_leaderboard_handler,
        crate::handlers::user::update_fcm_token::update_fcm_token_handler,
        crate::handlers::user::renew_token::renew_token_handler,
        crate::handlers::user::referral::use_referral_code_handler,
        crate::handlers::user::update::update_user_handler,
        crate::handlers::user::referral::create_special_code_handler,
        crate::handlers::user::admin_login::admin_signup_handler,
        crate::handlers::user::admin_login::admin_generate_otp,
        crate::handlers::user::admin_login::admin_login_handler
    ),
    components(
        schemas(
            crate::models::CreateUserReq,
            crate::models::LoginRequest,
            crate::models::CheckOtpReq,
            crate::models::FcmTokenReqBody,
            crate::models::RenewTokenReqBody,
            crate::models::ReferralCodeReqBody,
            crate::models::SpecialCodeReqBody,
            crate::models::UpdateUserReq,
            crate::models::AdminSignupRequest,

            crate::models::GenericResponse,
            crate::models::LoginResponse,
            crate::models::LeaderboardResponse,
            crate::models::UpdateUserResponse,
            crate::models::AdminLoginResponse,

            crate::models::user::User,
            crate::models::user::AdminUser,
            crate::models::user::LeaderboardData,
            crate::models::wallet::Money,

            crate::models::SocialLoginScheme,
            crate::models::user::LoginScheme,

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
