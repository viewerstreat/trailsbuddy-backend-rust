use dotenvy::dotenv;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;

#[tokio::main]
async fn main() {
    // create default env filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or("trailsbuddy_backend_rust=debug".into());

    // initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // import .env file
    dotenv().ok();

    // read the port number from env variable
    let port = std::env::var("PORT").unwrap_or_default();
    let port = port.parse::<u16>().unwrap_or(3000);
    // build the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // create the app instance
    let app = app::build_app();
    tracing::debug!("Starting the app in: {addr}");
    // start serving the app in the socket address
    axum::Server::bind(&addr).serve(app).await.unwrap();
}
