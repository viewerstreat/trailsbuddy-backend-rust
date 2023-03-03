use axum::{
    body::{boxed, Body},
    http::{header, HeaderValue},
    routing::{get, post, IntoMakeService},
    Router,
};
use mockall_double::double;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
    ServiceBuilderExt,
};

use crate::{
    constants::REQUEST_TIMEOUT_SECS,
    handlers::{
        clips::get_clips_handler,
        default::default_route_handler,
        global_404::global_404_handler,
        notification::get_noti::get_noti_handler,
        user::{
            check_otp::check_otp_handler, create::create_user_handler,
            get_leaderboard::get_leaderboard_handler, login::login_handler,
            renew_token::renew_token_handler, update::update_user_handler,
            update_fcm_token::update_fcm_token_handler, verify::verify_user_handler,
        },
    },
};

#[double]
use crate::database::AppDatabase;

/// Initializes the app with all routes and middlewares
pub async fn build() -> IntoMakeService<Router> {
    tracing::debug!("Initializing the app");
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
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    let db_client = Arc::new(db_client);

    let user_route = Router::new()
        .route("/verify", get(verify_user_handler))
        .route("/getLeaderboard", get(get_leaderboard_handler))
        .route("/login", post(login_handler))
        .route("/create", post(create_user_handler))
        .route("/checkOtp", get(check_otp_handler))
        .route("/updateFcmToken", post(update_fcm_token_handler))
        .route("/renewToken", post(renew_token_handler))
        .route("/update", post(update_user_handler));
    let clip_route = Router::new().route("/", get(get_clips_handler));
    let noti_route = Router::new().route("/", get(get_noti_handler));

    let api_route = Router::new()
        .nest("/user", user_route)
        .nest("/notification", noti_route)
        .nest("/clip", clip_route);

    // create the app instance with all routes and middleware
    let app: Router<(), Body> = Router::new()
        .route("/", get(default_route_handler))
        .nest("/api/v1", api_route)
        .layer(middleware)
        .fallback(global_404_handler)
        .with_state(db_client);
    // return the IntoMakeService instance
    app.into_make_service()
}
