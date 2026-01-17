//! MFBP - MindFry Binary Protocol
//!
//! A custom TCP binary protocol for high-performance communication
//! with MindFry Cognitive Database.
//!
//! ## Protocol Design
//!
//! - **Transport:** TCP
//! - **Encoding:** Binary (little-endian)
//! - **Frame:** `[u32 length][u8 opcode][payload...]`
//!
//! ## OpCode Ranges
//!
//! - `0x10-0x1F`: Lineage operations
//! - `0x20-0x2F`: Bond operations
//! - `0x30-0x3F`: Query operations
//! - `0x40-0x4F`: System operations
//! - `0x50-0x5F`: Stream operations

#![allow(missing_docs)]

mod codec;
mod handler;
mod message;
mod opcodes;

pub use codec::{MfbpCodec, MfbpError};
pub use handler::CommandHandler;
pub use message::*;
pub use opcodes::*;
