use std::{net::SocketAddr, sync::Arc};

use database::AppDatabase;
use dotenvy::dotenv;
use jobs::spawn_all_jobs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod app;
pub mod constants;
pub mod database;
pub mod handlers;
pub mod jobs;
pub mod jwt;
pub mod models;
pub mod swagger;
pub mod utils;

pub async fn start_web_server() {
    // import .env file
    dotenv().ok();
    // create database client
    let db_client = AppDatabase::new()
        .await
        .expect("Unable to accquire database client");
    let db_client = Arc::new(db_client);
    initialize_logging();
    spawn_all_jobs(db_client.clone());
    start_server(db_client).await;
}

fn initialize_logging() {
    // create default env filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or("trailsbuddy_backend_rust=debug".into());

    // initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}

async fn start_server(db_client: Arc<AppDatabase>) {
    // read the port number from env variable
    let port = std::env::var("PORT").unwrap_or_default();
    let port = port.parse::<u16>().unwrap_or(3000);
    // build the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // create the app instance
    let app = app::build_app(db_client);
    tracing::debug!("Starting the app in: {addr}");
    // start serving the app in the socket address
    axum::Server::bind(&addr).serve(app).await.unwrap();
}
