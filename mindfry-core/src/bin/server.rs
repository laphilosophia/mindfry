//! MindFry Server - TCP server binary
//!
//! Standalone server for MindFry Cognitive Database.
//! Speaks MFBP (MindFry Binary Protocol) over TCP.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin mindfry-server -- --port 9527
//! ```

use std::sync::{Arc, RwLock};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{Level, error, info, warn};
use tracing_subscriber::FmtSubscriber;

use mindfry::protocol::{CommandHandler, MfbpCodec, Request};
use mindfry::{MindFry, MindFryConfig};

/// Default server port (MFBP)
const DEFAULT_PORT: u16 = 9527;

/// Maximum frame size (16 MB)
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// Server configuration
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: DEFAULT_PORT,
            max_connections: 1024,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Banner
    println!();
    println!("  ╔═══════════════════════════════════════════════════════════╗");
    println!("  ║  🧠🔥 MindFry - The World's First Ephemeral Graph Database  ║");
    println!("  ║                     COGNITIVE DB ENGINE                    ║");
    println!("  ╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Parse args (simple for now)
    let server_config = ServerConfig::default();

    // Create MindFry instance (shared across connections)
    let db_config = MindFryConfig::default();
    let db = Arc::new(RwLock::new(MindFry::with_config(db_config)));

    info!(
        "Psyche Arena capacity: {} lineages",
        db.read().unwrap().psyche.capacity()
    );
    info!(
        "Bond Graph capacity: {} bonds",
        db.read().unwrap().bonds.len()
    );

    // Start TCP server
    let addr = format!("{}:{}", server_config.host, server_config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🌐 MFBP Server listening on {}", addr);
    info!(
        "Ready to accept connections (max: {})",
        server_config.max_connections
    );

    // Accept loop
    loop {
        match listener.accept().await {
            Ok((socket, peer)) => {
                info!("📥 New connection from {}", peer);

                // Clone Arc for the handler
                let db_clone = Arc::clone(&db);

                // Spawn connection handler
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, db_clone).await {
                        error!("Connection error: {}", e);
                    }
                    info!("📤 Connection closed: {}", peer);
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(
    mut socket: TcpStream,
    db: Arc<RwLock<MindFry>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut handler = CommandHandler::new(db);
    let mut buffer = vec![0u8; 4096];
    let mut read_buf = Vec::new();

    loop {
        // Read data into buffer
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            // Connection closed
            return Ok(());
        }

        // Append to read buffer
        read_buf.extend_from_slice(&buffer[..n]);

        // Try to parse complete frames
        while read_buf.len() >= 5 {
            // Peek at frame length
            let frame_len =
                u32::from_le_bytes([read_buf[0], read_buf[1], read_buf[2], read_buf[3]]) as usize;

            if frame_len > MAX_FRAME_SIZE {
                warn!("Frame too large: {} bytes", frame_len);
                return Err("Frame too large".into());
            }

            let total_len = 4 + frame_len;
            if read_buf.len() < total_len {
                // Need more data
                break;
            }

            // Extract frame
            let frame: Vec<u8> = read_buf.drain(..total_len).collect();

            // Decode request
            match MfbpCodec::decode_request(&frame) {
                Ok(request) => {
                    // Log request type
                    log_request(&request);

                    // Handle request
                    let response = handler.handle(request);

                    // Encode response
                    let response_bytes = MfbpCodec::encode_response(&response);

                    // Send response
                    socket.write_all(&response_bytes).await?;
                }
                Err(e) => {
                    warn!("Failed to decode request: {}", e);
                    // Send error response
                    let error_response = mindfry::protocol::Response::Error {
                        code: mindfry::protocol::ErrorCode::MalformedPayload,
                        message: format!("Failed to decode: {}", e),
                    };
                    let response_bytes = MfbpCodec::encode_response(&error_response);
                    socket.write_all(&response_bytes).await?;
                }
            }
        }
    }
}

/// Log request type for debugging
fn log_request(request: &Request) {
    match request {
        Request::Ping => info!("  → PING"),
        Request::Stats => info!("  → STATS"),
        Request::LineageCreate { id, .. } => info!("  → LINEAGE.CREATE '{}'", id),
        Request::LineageGet { id } => info!("  → LINEAGE.GET '{}'", id),
        Request::LineageStimulate { id, delta } => {
            info!("  → LINEAGE.STIMULATE '{}' +{}", id, delta)
        }
        Request::LineageForget { id } => info!("  → LINEAGE.FORGET '{}'", id),
        Request::BondConnect { source, target, .. } => {
            info!("  → BOND.CONNECT '{}' ↔ '{}'", source, target)
        }
        Request::QueryConscious { .. } => info!("  → QUERY.CONSCIOUS"),
        Request::QueryTopK { k } => info!("  → QUERY.TOP_K({})", k),
        Request::QueryTrauma { min_rigidity } => info!("  → QUERY.TRAUMA(≥{})", min_rigidity),
        Request::Snapshot { name } => info!("  → SYS.SNAPSHOT '{}'", name),
        Request::Freeze { frozen } => {
            info!("  → SYS.{}", if *frozen { "FREEZE" } else { "THAW" })
        }
        _ => info!("  → {:?}", request.opcode()),
    }
}
