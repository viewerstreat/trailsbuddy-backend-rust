use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use dotenvy::dotenv;
use serde::Serialize;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    println!("Initializing the app...");
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();
    // build the app with route
    let app = Router::new().route("/", get(root));
    // .route("/users", post(create_user));
    let port = env::var("PORT").unwrap();
    let port = port.parse::<u16>().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("listening on {}", addr);

    // start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Serialize)]
struct RootResponse {
    success: bool,
    message: String,
}

// basic handler that responds with a static string
async fn root() -> impl IntoResponse {
    let response = RootResponse {
        success: true,
        message: "Server is running".to_owned(),
    };

    (StatusCode::OK, Json(response))
}
