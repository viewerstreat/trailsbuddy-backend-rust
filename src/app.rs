use axum::{
    body::boxed,
    extract::DefaultBodyLimit,
    http::{header, HeaderValue},
    routing::{get, post, IntoMakeService},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
    ServiceBuilderExt,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{constants::*, database::AppDatabase, handlers::*, swagger::ApiDoc};

/// Initializes the app with all routes and middlewares
pub fn build_app_routes(db_client: Arc<AppDatabase>) -> Router {
    let root_route = Router::new()
        .route("/ping", get(ping_handler))
        .route("/tempApiGetToken", get(temp_api_get_token))
        .route("/tempApiGetOtp", get(temp_api_get_otp));
    let user_route = Router::new()
        .route("/create", post(create_user_handler))
        .route("/login", post(login_handler))
        .route("/verify", get(verify_user_handler))
        .route("/checkOtp", get(check_otp_handler))
        .route("/getLeaderboard", get(get_leaderboard_handler))
        .route("/updateFcmToken", post(update_fcm_token_handler))
        .route("/renewToken", post(renew_token_handler))
        .route("/useReferralCode", post(use_referral_code_handler))
        .route("/update", post(update_user_handler));
    let admin_route = Router::new()
        .route("/signup", post(admin_signup_handler))
        .route("/generateOtp", get(admin_generate_otp))
        .route("/login", post(admin_login_handler))
        .route(
            "/createSpecialReferralCode",
            post(create_special_code_handler),
        );
    let clip_route = Router::new()
        .route("/", get(get_clips_handler))
        .route("/", post(create_clip_handler))
        .route("/addView", post(add_clip_view_handler));
    let movie_route = Router::new()
        .route("/", get(get_movie_handler))
        .route("/", post(create_movie_handler))
        .route("/details", get(movie_details_handler))
        .route("/addView", post(add_movie_view_handler))
        .route("/isLikedByMe", get(is_liked_by_me_handler));
    let fav_route = Router::new()
        .route("/", post(add_favourite_handler))
        .route("/", get(get_favourite_handler));
    let contest_route = Router::new()
        .route("/", post(create_contest_handler))
        .route("/", get(get_contest_handler))
        .route("/activate", post(activate_contest_handler))
        .route("/inActivate", post(inactivate_contest_handler));
    let question_route = Router::new()
        .route("/", post(create_question_handler))
        .route("/", get(get_question_handler))
        .route("/delete", post(delete_question_handler))
        .route("/update", post(update_question_handler));
    let noti_route = Router::new()
        .route("/", get(get_noti_handler))
        .route("/clear", post(clear_noti_handler))
        .route("/clearall", post(clear_all_noti_handler))
        .route("/markRead", post(mark_read_noti_handler))
        .route("/markAllRead", post(mark_all_read_noti_handler))
        .route("/createBroadcast", post(create_broadcast_noti_handler));
    let upload_route = Router::new()
        .route("/single", post(upload_handler))
        .route("/multipart/uploadPart", post(upload_part_multipart_handler))
        .layer(DefaultBodyLimit::max(MULTIPART_BODY_LIMIT))
        .route("/multipart/initiate", get(create_multipart_handler))
        .route("/multipart/finish", post(complete_multipart_handler));
    let wallet_route = Router::new()
        .route("/getBalance", get(get_bal_handler))
        .route("/addBalanceInit", post(add_bal_init_handler))
        .route("/addBalanceEnd", post(add_bal_end_handler))
        .route("/withdrawBalInit", post(withdraw_bal_init_handler))
        .route("/withdrawBalanceEnd", post(withdraw_bal_end_handler))
        .route("/payContest", post(pay_contest_handler));
    let play_tracker_route = Router::new()
        .route("/", get(get_play_tracker_handler))
        .route("/start", post(start_play_tracker_handler))
        .route("/getNextQues", get(get_next_ques_handler))
        .route("/answer", post(answer_play_tracker_handler))
        .route("/finish", post(finish_play_tracker_handler));

    let api_route = Router::new()
        .nest("/", root_route)
        .nest("/user", user_route)
        .nest("/notification", noti_route)
        .nest("/movie", movie_route)
        .nest("/clip", clip_route)
        .nest("/favourite", fav_route)
        .nest("/question", question_route)
        .nest("/contest", contest_route)
        .nest("/wallet", wallet_route)
        .nest("/playTracker", play_tracker_route)
        .nest("/upload", upload_route)
        .nest("/admin", admin_route);

    // create a response header layer for middleware
    let server_header_value = HeaderValue::from_static("trailsbuddy-backend-rust");
    let set_response_header_layer =
        SetResponseHeaderLayer::if_not_present(header::SERVER, server_header_value);
    // create the trace layer for middleware
    let trace_layer = TraceLayer::new_for_http();
    // create the cors layer for middleware
    let cors_layer = CorsLayer::permissive();
    // create the timeout layer for middleware
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS));
    // combine all middlewares with ServiceBuilder
    let middleware = ServiceBuilder::new()
        .layer(timeout_layer)
        .layer(cors_layer)
        .layer(set_response_header_layer)
        .map_response_body(boxed)
        .layer(trace_layer)
        .compression()
        .into_inner();

    // create the app instance with all routes and middleware
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(default_route_handler))
        .nest("/api/v1", api_route)
        .layer(middleware)
        .fallback(global_404_handler)
        .with_state(db_client);
    app
}

/// Initializes the app instance
pub fn build_app(db_client: Arc<AppDatabase>) -> IntoMakeService<Router> {
    tracing::debug!("Initializing the app");
    let app = build_app_routes(db_client);
    // return the IntoMakeService instance
    app.into_make_service()
}
