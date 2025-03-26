use mimic_rs::MockServer;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialization of the logger
    tracing_subscriber::fmt::init();

    // Get the port from the arguments or use the default 8080
    let port = env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u16>().ok())
        .unwrap_or(8080);

    // Get the resources directory from the arguments or use the default "./resources"
    let resources_dir = env::args()
        .nth(2)
        .unwrap_or_else(|| "./resources".to_string());

    info!(
        "MockServer is starting on port {} with resources in {}",
        port, resources_dir
    );

    // Create and start the server
    let server = MockServer::new(resources_dir);
    server.start(port).await?;

    Ok(())
}
