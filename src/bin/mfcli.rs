//! MindFry CLI Client
//!
//! Simple command-line client for testing MFBP protocol.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin mfcli -- ping
//! cargo run --bin mfcli -- stats
//! cargo run --bin mfcli -- create fire 0.8
//! cargo run --bin mfcli -- get fire
//! ```

use std::io::{Read, Write};
use std::net::TcpStream;

use mindfry::protocol::{MfbpCodec, Request};

const DEFAULT_HOST: &str = "127.0.0.1:9527";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];
    let request = match command.as_str() {
        "ping" => Request::Ping,
        "stats" => Request::Stats,
        "create" => {
            if args.len() < 4 {
                eprintln!("Usage: mfcli create <id> <energy>");
                return Ok(());
            }
            Request::LineageCreate {
                id: args[2].clone(),
                energy: args[3].parse()?,
                threshold: 0.5,
                decay_rate: 0.001,
            }
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("Usage: mfcli get <id>");
                return Ok(());
            }
            Request::LineageGet {
                id: args[2].clone(),
            }
        }
        "stimulate" => {
            if args.len() < 4 {
                eprintln!("Usage: mfcli stimulate <id> <delta>");
                return Ok(());
            }
            Request::LineageStimulate {
                id: args[2].clone(),
                delta: args[3].parse()?,
            }
        }
        "forget" => {
            if args.len() < 3 {
                eprintln!("Usage: mfcli forget <id>");
                return Ok(());
            }
            Request::LineageForget {
                id: args[2].clone(),
            }
        }
        "connect" => {
            if args.len() < 5 {
                eprintln!("Usage: mfcli connect <source> <target> <strength>");
                return Ok(());
            }
            Request::BondConnect {
                source: args[2].clone(),
                target: args[3].clone(),
                strength: args[4].parse()?,
            }
        }
        "neighbors" => {
            if args.len() < 3 {
                eprintln!("Usage: mfcli neighbors <id>");
                return Ok(());
            }
            Request::BondNeighbors {
                id: args[2].clone(),
            }
        }
        "conscious" => {
            let min_energy = if args.len() > 2 {
                args[2].parse()?
            } else {
                0.5
            };
            Request::QueryConscious { min_energy }
        }
        "topk" => {
            let k = if args.len() > 2 { args[2].parse()? } else { 10 };
            Request::QueryTopK { k }
        }
        "trauma" => {
            let min_rigidity = if args.len() > 2 {
                args[2].parse()?
            } else {
                0.8
            };
            Request::QueryTrauma { min_rigidity }
        }
        "freeze" => Request::Freeze { frozen: true },
        "thaw" => Request::Freeze { frozen: false },
        "snapshot" => {
            let name = if args.len() > 2 {
                args[2].clone()
            } else {
                "manual".into()
            };
            Request::Snapshot { name }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            return Ok(());
        }
    };

    // Connect to server
    let mut stream = TcpStream::connect(DEFAULT_HOST)?;
    println!("Connected to {}", DEFAULT_HOST);

    // Send request
    let request_bytes = MfbpCodec::encode_request(&request);
    stream.write_all(&request_bytes)?;
    println!("Sent: {:?}", request.opcode());

    // Read response
    let mut response_buf = vec![0u8; 65536];
    let n = stream.read(&mut response_buf)?;

    if n == 0 {
        eprintln!("Server closed connection");
        return Ok(());
    }

    // Decode and display response
    // Note: We're reading raw response bytes, need to parse
    println!("\nResponse ({} bytes):", n);

    // Simple response parsing (peek at opcode)
    if n >= 5 {
        let opcode = response_buf[4];
        match opcode {
            0xF0 => {
                println!("✅ OK");
                // Try to parse response data
                parse_response_data(&response_buf[..n]);
            }
            0xF1 => {
                println!("❌ ERROR");
                // Error code is at byte 5
                if n > 5 {
                    println!("   Code: 0x{:02X}", response_buf[5]);
                }
            }
            _ => {
                println!("Unknown response opcode: 0x{:02X}", opcode);
            }
        }
    }

    Ok(())
}

fn parse_response_data(data: &[u8]) {
    if data.len() < 6 {
        return;
    }

    let data_type = data[5];
    match data_type {
        0x00 => println!("   Type: Ack"),
        0x01 => println!("   Type: Pong 🏓"),
        0x02 => {
            println!("   Type: Lineage");
            // Parse lineage info if we have enough data
        }
        0x03 => {
            println!("   Type: Lineages[]");
            // Parse list
        }
        0x04 => println!("   Type: Neighbors[]"),
        0x05 => {
            println!("   Type: Stats");
            // Parse stats - starts at byte 6
            if data.len() >= 26 {
                let lineage_count = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
                let bond_count = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);
                let conscious_count = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);
                println!("   Lineages: {}", lineage_count);
                println!("   Bonds: {}", bond_count);
                println!("   Conscious: {}", conscious_count);
            }
        }
        0x06 => println!("   Type: SnapshotCreated"),
        _ => println!("   Unknown data type: 0x{:02X}", data_type),
    }
}

fn print_usage() {
    println!("MindFry CLI Client");
    println!();
    println!("Usage: mfcli <command> [args...]");
    println!();
    println!("Commands:");
    println!("  ping                          Test connection");
    println!("  stats                         Get database statistics");
    println!("  create <id> <energy>          Create a lineage");
    println!("  get <id>                      Get lineage info");
    println!("  stimulate <id> <delta>        Stimulate a lineage");
    println!("  forget <id>                   Forget (soft-delete) a lineage");
    println!("  connect <src> <tgt> <str>     Create a bond");
    println!("  neighbors <id>                Get neighbors of a lineage");
    println!("  conscious [min_energy]        Query conscious lineages");
    println!("  topk [k]                      Get top K lineages");
    println!("  trauma [min_rigidity]         Query traumatized lineages");
    println!("  freeze                        Freeze decay engine");
    println!("  thaw                          Unfreeze decay engine");
    println!("  snapshot [name]               Take a snapshot");
}
