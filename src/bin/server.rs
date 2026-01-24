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
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use mindfry::persistence::{AkashicConfig, AkashicStore};
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
    println!("  ║                  Memory with a Conscience                 ║");
    println!("  ║                    COGNITIVE DB ENGINE                    ║");
    println!("  ╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Parse args (simple for now)
    let server_config = ServerConfig::default();

    // ═══════════════════════════════════════════════════════════════
    // INITIALIZATION SEQUENCE (Network-first for zero delay)
    // ═══════════════════════════════════════════════════════════════

    println!("  ┌─ Initialization ─────────────────────────────────────────┐");

    // Step 1: Mount Storage
    print!("  │ 📁 Mounting Akashic Records...");
    std::io::Write::flush(&mut std::io::stdout())?;
    let store_config = AkashicConfig::default();
    let store = match AkashicStore::open(store_config) {
        Ok(s) => {
            println!(" ✓");
            Arc::new(s)
        }
        Err(e) => {
            println!(" ✗");
            error!("Failed to open storage: {}", e);
            return Err(e.into());
        }
    };

    // Step 2: Initialize Psyche Arena (empty)
    print!("  │ 🧠 Initializing Psyche Arena...");
    std::io::Write::flush(&mut std::io::stdout())?;
    let db_config = MindFryConfig::default();
    let db = MindFry::with_config(db_config).with_store(Arc::clone(&store));
    println!(" ✓");

    // Step 3: Bind Network (before resurrection for zero delay)
    print!("  │ 🌐 Binding network interface...");
    std::io::Write::flush(&mut std::io::stdout())?;
    let addr = format!("{}:{}", server_config.host, server_config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!(" ✓ ({})", addr);

    // Wrap DB in Arc<RwLock> for sharing
    let db = Arc::new(RwLock::new(db));

    // Step 4: Setup warmup tracker
    let warmup = mindfry::stability::WarmupTracker::new();

    // Check if resurrection is needed
    let has_snapshot = store
        .list_snapshots()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if has_snapshot {
        print!("  │ 🔄 Resurrection...");
        std::io::Write::flush(&mut std::io::stdout())?;
        warmup.begin_resurrection();
        println!(" (async)");

        // Spawn async resurrection
        let db_clone = Arc::clone(&db);
        let warmup_clone = warmup.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();

            // Perform resurrection
            let result = {
                let mut db = db_clone.write().unwrap();
                db.resurrect()
            };

            match result {
                Ok(true) => {
                    // Bootstrap system lineages after resurrection
                    {
                        let mut db = db_clone.write().unwrap();
                        db.bootstrap_system_lineages();
                    }
                    info!("✅ Resurrection complete in {:?}", start.elapsed());
                }
                Ok(false) => {
                    info!("🌱 No snapshot found, genesis mode");
                }
                Err(e) => {
                    warn!("⚠️ Resurrection failed: {}", e);
                }
            }

            warmup_clone.mark_ready();
        });
    } else {
        print!("  │ 🌱 Genesis mode...");
        std::io::Write::flush(&mut std::io::stdout())?;
        // Bootstrap system lineages for fresh start
        {
            let mut db = db.write().unwrap();
            db.bootstrap_system_lineages();
        }
        println!(" ✓");
    }

    println!("  └────────────────────────────────────────────────────────────┘");
    println!();

    // Summary
    info!(
        "Ready | {} lineages | {} bonds | max {} connections | warmup: {:?}",
        db.read().unwrap().psyche.len(),
        db.read().unwrap().bonds.len(),
        server_config.max_connections,
        warmup.state()
    );

    // ═══════════════════════════════════════════════════════════════
    // MAIN LOOP WITH GRACEFUL SHUTDOWN
    // ═══════════════════════════════════════════════════════════════

    let shutdown_result = tokio::select! {
        result = accept_loop(listener, Arc::clone(&db)) => {
            // Accept loop returned (error or explicit stop)
            result
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown signal received (Ctrl+C)");
            Ok(mindfry::stability::ShutdownReason::Signal { signal: 2 }) // SIGINT
        }
    };

    // ═══════════════════════════════════════════════════════════════
    // GRACEFUL SHUTDOWN SEQUENCE
    // ═══════════════════════════════════════════════════════════════

    match shutdown_result {
        Ok(reason) => {
            info!("📝 Recording shutdown experience: {}", reason.description());

            // Take final snapshot before shutdown
            {
                let db_guard = db.read().unwrap();
                if let Some(ref store) = db_guard.store {
                    match store.take_snapshot(
                        Some("pre-shutdown"),
                        &db_guard.psyche,
                        &db_guard.strata,
                        &db_guard.bonds,
                        Some(&db_guard.cortex),
                        mindfry::persistence::PhysicsSnapshot::default(),
                    ) {
                        Ok(meta) => info!("💾 Pre-shutdown snapshot saved: {}", meta.id),
                        Err(e) => warn!("⚠️ Failed to save shutdown snapshot: {}", e),
                    }
                }
            }

            info!("😴 MindFry going to sleep... Goodbye!");
        }
        Err(e) => {
            error!("💔 Server error: {}", e);
        }
    }

    Ok(())
}

/// Accept loop - runs until error or shutdown
async fn accept_loop(
    listener: TcpListener,
    db: Arc<RwLock<MindFry>>,
) -> Result<mindfry::stability::ShutdownReason, Box<dyn std::error::Error + Send + Sync>> {
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
                return Err(e.into());
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
        Request::LineageGet { id, flags } => {
            info!("  → LINEAGE.GET '{}' [flags:0x{:02X}]", id, flags)
        }
        Request::LineageStimulate { id, delta, flags } => {
            info!(
                "  → LINEAGE.STIMULATE '{}' +{} [flags:0x{:02X}]",
                id, delta, flags
            )
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
