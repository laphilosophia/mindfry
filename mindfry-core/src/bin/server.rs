//! MindFry Server - TCP server binary
//!
//! Standalone server for MindFry Cognitive DB.

use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use mindfry::{MindFry, MindFryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🧠🔥 MindFry - The World's First Ephemeral Graph Database");
    info!("Starting server...");

    // Create MindFry instance
    let config = MindFryConfig::default();
    let db = MindFry::with_config(config);

    info!(
        "Psyche Arena: {} lineages capacity",
        db.psyche.capacity()
    );

    // Start TCP server
    let addr = "127.0.0.1:6379"; // TODO: Make configurable
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    // TODO: Accept connections and handle MFBP protocol
    loop {
        let (socket, peer) = listener.accept().await?;
        info!("New connection from {}", peer);

        // TODO: Spawn handler task
        drop(socket); // Placeholder
    }
}
