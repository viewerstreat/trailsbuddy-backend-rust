use axum::{
    body::{boxed, Body},
    extract::DefaultBodyLimit,
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
    constants::*,
    handlers::{
        clip::{
            add_view::add_clip_view_handler, create::create_clip_handler,
            get_clip::get_clips_handler,
        },
        default::default_route_handler,
        favourite::{create::add_favourite_handler, get::get_favourite_handler},
        global_404::global_404_handler,
        movie::{
            add_view::add_movie_view_handler, create::create_movie_handler,
            details::movie_details_handler, get_movie::get_movie_handler,
            liked_by_me::is_liked_by_me_handler,
        },
        notification::{
            clear_noti::{clear_all_noti_handler, clear_noti_handler},
            get_noti::get_noti_handler,
            mark_read::{mark_all_read_noti_handler, mark_read_noti_handler},
        },
        ping::ping_handler,
        temp_api::{temp_api_get_otp, temp_api_get_token},
        upload::single::upload_handler,
        user::{
            check_otp::check_otp_handler, create::create_user_handler,
            get_leaderboard::get_leaderboard_handler, login::login_handler,
            renew_token::renew_token_handler, update::update_user_handler,
            update_fcm_token::update_fcm_token_handler, verify::verify_user_handler,
        },
        wallet::{
            add_bal::{add_bal_end_handler, add_bal_init_handler},
            get_bal::get_bal_handler,
            withdraw_bal::{withdraw_bal_end_handler, withdraw_bal_init_handler},
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

    let root_route = Router::new()
        .route("/ping", get(ping_handler))
        .route("/tempApiGetToken", get(temp_api_get_token))
        .route("/tempApiGetOtp", get(temp_api_get_otp));
    let user_route = Router::new()
        .route("/verify", get(verify_user_handler))
        .route("/getLeaderboard", get(get_leaderboard_handler))
        .route("/login", post(login_handler))
        .route("/create", post(create_user_handler))
        .route("/checkOtp", get(check_otp_handler))
        .route("/updateFcmToken", post(update_fcm_token_handler))
        .route("/renewToken", post(renew_token_handler))
        .route("/update", post(update_user_handler));
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
    let noti_route = Router::new()
        .route("/", get(get_noti_handler))
        .route("/clear", post(clear_noti_handler))
        .route("/clearall", post(clear_all_noti_handler))
        .route("/markRead", post(mark_read_noti_handler))
        .route("/markAllRead", post(mark_all_read_noti_handler));
    let upload_route = Router::new()
        .route("/single", post(upload_handler))
        .layer(DefaultBodyLimit::max(MULTIPART_BODY_LIMIT));
    let wallet_route = Router::new()
        .route("/getBalance", get(get_bal_handler))
        .route("/addBalanceInit", post(add_bal_init_handler))
        .route("/addBalanceEnd", post(add_bal_end_handler))
        .route("/withdrawBalInit", post(withdraw_bal_init_handler))
        .route("/withdrawBalanceEnd", post(withdraw_bal_end_handler));

    let api_route = Router::new()
        .nest("/", root_route)
        .nest("/user", user_route)
        .nest("/notification", noti_route)
        .nest("/movie", movie_route)
        .nest("/clip", clip_route)
        .nest("/favourite", fav_route)
        .nest("/wallet", wallet_route)
        .nest("/upload", upload_route);

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
