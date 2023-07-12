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
        crate::handlers::user::admin_login::admin_login_handler,
        crate::handlers::clip::get_clip::get_clips_handler,
        crate::handlers::clip::create::create_clip_handler,
        crate::handlers::clip::add_view::add_clip_view_handler,
        crate::handlers::movie::get_movie::get_movie_handler,
        crate::handlers::movie::create::create_movie_handler,
        crate::handlers::movie::details::movie_details_handler,
        crate::handlers::movie::add_view::add_movie_view_handler,
        crate::handlers::movie::liked_by_me::is_liked_by_me_handler,
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
            crate::models::CreateClipReqBody,
            crate::models::ClipAddViewReqBody,
            crate::models::CreateMovieReqBody,
            crate::models::MovieAddViewReqBody,

            crate::models::GenericResponse,
            crate::models::LoginResponse,
            crate::models::LeaderboardResponse,
            crate::models::UpdateUserResponse,
            crate::models::AdminLoginResponse,
            crate::models::GetClipResponse,
            crate::models::ClipResponse,
            crate::models::AddViewResponse,
            crate::models::MovieResponse,
            crate::models::MovieDetailResponse,
            crate::models::MovieLikedResponse,

            crate::models::User,
            crate::models::AdminUser,
            crate::models::LeaderboardData,
            crate::models::Money,
            crate::models::WrapDocument,
            crate::models::ClipRespData,
            crate::models::ClipProps,
            crate::models::LikesEntry,
            crate::models::ViewsEntry,
            crate::models::MovieProps,
            crate::models::MovieRespData,
            crate::models::MovieDetails,

            crate::models::SocialLoginScheme,
            crate::models::LoginScheme,

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
