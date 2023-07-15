use trailsbuddy_backend_rust::start_web_server;

#[tokio::main]
async fn main() {
    start_web_server().await;
}
