//! MFBP Message Structures
//!
//! Request and response message types for the protocol.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use super::OpCode;

// ═══════════════════════════════════════════════════════════════
// QUERY FLAGS (Executive Override)
// ═══════════════════════════════════════════════════════════════

bitflags! {
    /// Query flags for controlling access behavior
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct QueryFlags: u8 {
        /// No special flags (default behavior)
        const NONE = 0x00;
        /// Bypass Cortex mood and Antagonism filters
        const BYPASS_FILTERS = 0x01;
        /// Return REPRESSED status instead of hiding
        const INCLUDE_REPRESSED = 0x02;
        /// Don't stimulate on read (no observer effect)
        const NO_SIDE_EFFECTS = 0x04;
        /// All flags combined (forensic/god mode)
        const FORENSIC = 0x07;
    }
}

bitflags! {
    /// Stimulate flags for controlling propagation behavior
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct StimulateFlags: u8 {
        /// Normal behavior - propagate to neighbors
        const NONE = 0x00;
        /// Surgical mode - don't propagate to neighbors
        const NO_PROPAGATE = 0x01;
    }
}

/// Status of lineage lookup result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LineageStatus {
    /// Normal success - lineage found and accessible
    Found = 0,
    /// Lineage does not exist in storage
    NotFound = 1,
    /// Lineage exists but is repressed by Antagonism
    Repressed = 2,
    /// Lineage is in retention buffer (scheduled for GC)
    Dormant = 3,
}

// ═══════════════════════════════════════════════════════════════
// REQUEST MESSAGES
// ═══════════════════════════════════════════════════════════════

/// A request message from client to server
#[derive(Debug, Clone)]
pub enum Request {
    // Lineage
    LineageCreate {
        id: String,
        energy: f32,
        threshold: f32,
        decay_rate: f32,
    },
    LineageGet {
        id: String,
        /// Query flags for access control (default: NONE)
        flags: u8,
    },
    LineageStimulate {
        id: String,
        delta: f32,
        /// Stimulate flags (default: auto-propagate)
        flags: u8,
    },
    LineageForget {
        id: String,
    },
    LineageTouch {
        id: String,
    },

    // Bond
    BondConnect {
        source: String,
        target: String,
        strength: f32,
        /// Polarity: +1=Synergy, 0=Neutral, -1=Antagonism
        polarity: i8,
    },
    BondReinforce {
        source: String,
        target: String,
        delta: f32,
    },
    BondSever {
        source: String,
        target: String,
    },
    BondNeighbors {
        id: String,
    },

    // Query
    QueryConscious {
        min_energy: f32,
    },
    QueryTopK {
        k: u32,
    },
    QueryTrauma {
        min_rigidity: f32,
    },
    QueryPattern {
        pattern: String,
    },

    // System
    Ping,
    Stats,
    Snapshot {
        name: String,
    },
    Restore {
        name: String,
    },
    Freeze {
        frozen: bool,
    },
    PhysicsTune {
        param: u8,
        value: f32,
    },
    /// Set Cortex mood (external override)
    MoodSet {
        mood: f32,
    },

    // Stream
    Subscribe {
        events_mask: u32,
    },
    Unsubscribe,
}

impl Request {
    /// Get the OpCode for this request
    pub fn opcode(&self) -> OpCode {
        match self {
            Self::LineageCreate { .. } => OpCode::LineageCreate,
            Self::LineageGet { .. } => OpCode::LineageGet,
            Self::LineageStimulate { .. } => OpCode::LineageStimulate,
            Self::LineageForget { .. } => OpCode::LineageForget,
            Self::LineageTouch { .. } => OpCode::LineageTouch,
            Self::BondConnect { .. } => OpCode::BondConnect,
            Self::BondReinforce { .. } => OpCode::BondReinforce,
            Self::BondSever { .. } => OpCode::BondSever,
            Self::BondNeighbors { .. } => OpCode::BondNeighbors,
            Self::QueryConscious { .. } => OpCode::QueryConscious,
            Self::QueryTopK { .. } => OpCode::QueryTopK,
            Self::QueryTrauma { .. } => OpCode::QueryTrauma,
            Self::QueryPattern { .. } => OpCode::QueryPattern,
            Self::Ping => OpCode::SysPing,
            Self::Stats => OpCode::SysStats,
            Self::Snapshot { .. } => OpCode::SysSnapshot,
            Self::Restore { .. } => OpCode::SysRestore,
            Self::Freeze { .. } => OpCode::SysFreeze,
            Self::PhysicsTune { .. } => OpCode::PhysicsTune,
            Self::MoodSet { .. } => OpCode::SysMoodSet,
            Self::Subscribe { .. } => OpCode::StreamSubscribe,
            Self::Unsubscribe => OpCode::StreamUnsubscribe,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// RESPONSE MESSAGES
// ═══════════════════════════════════════════════════════════════

/// A response message from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Success with optional data
    Ok(ResponseData),

    /// Error with message
    Error { code: ErrorCode, message: String },

    /// Event notification
    Event(Event),
}

/// Response data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    /// Empty acknowledgment
    Ack,

    /// Pong response
    Pong,

    /// Single lineage lookup result (with status header)
    LineageResult(LineageResult),

    /// List of lineages
    Lineages(Vec<LineageInfo>),

    /// List of neighbors with bond strength
    Neighbors(Vec<NeighborInfo>),

    /// Database statistics
    Stats(StatsInfo),

    /// Snapshot created
    SnapshotCreated { name: String },
}

/// Lineage lookup result with status framing
/// Wire format: [status:u8] + [payload?]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageResult {
    /// Lookup status
    pub status: LineageStatus,
    /// Lineage info (only present if status == Found)
    pub info: Option<LineageInfo>,
}

/// Lineage information for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    pub id: String,
    pub energy: f32,
    pub threshold: f32,
    pub decay_rate: f32,
    pub rigidity: f32,
    pub is_conscious: bool,
    pub last_access_ms: u64,
}

/// Neighbor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborInfo {
    pub id: String,
    pub bond_strength: f32,
    pub is_learned: bool,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsInfo {
    pub lineage_count: usize,
    pub bond_count: usize,
    pub conscious_count: usize,
    pub total_energy: f32,
    pub is_frozen: bool,
    pub uptime_secs: u64,
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ErrorCode {
    /// Unknown error
    Unknown = 0x00,
    /// Invalid OpCode
    InvalidOpCode = 0x01,
    /// Malformed payload
    MalformedPayload = 0x02,
    /// Lineage not found
    LineageNotFound = 0x10,
    /// Lineage already exists
    LineageExists = 0x11,
    /// Bond not found
    BondNotFound = 0x20,
    /// Bond already exists
    BondExists = 0x21,
    /// Snapshot not found
    SnapshotNotFound = 0x30,
    /// Internal error
    Internal = 0xFF,
}

impl ErrorCode {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => Self::InvalidOpCode,
            0x02 => Self::MalformedPayload,
            0x10 => Self::LineageNotFound,
            0x11 => Self::LineageExists,
            0x20 => Self::BondNotFound,
            0x21 => Self::BondExists,
            0x30 => Self::SnapshotNotFound,
            0xFF => Self::Internal,
            _ => Self::Unknown,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// EVENTS
// ═══════════════════════════════════════════════════════════════

/// Event notification for subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    LineageCreated {
        id: String,
        energy: f32,
    },
    LineageStimulated {
        id: String,
        new_energy: f32,
        delta: f32,
    },
    LineageForgotten {
        id: String,
    },
    BondCreated {
        source: String,
        target: String,
        strength: f32,
    },
    BondSevered {
        source: String,
        target: String,
    },
    DecayTick {
        processed: usize,
        dead_count: usize,
    },
    SnapshotCreated {
        name: String,
    },
}

impl Event {
    /// Get the event mask bit for this event
    pub fn mask_bit(&self) -> u32 {
        match self {
            Self::LineageCreated { .. } => 1 << 0,
            Self::LineageStimulated { .. } => 1 << 1,
            Self::LineageForgotten { .. } => 1 << 2,
            Self::BondCreated { .. } => 1 << 3,
            Self::BondSevered { .. } => 1 << 4,
            Self::DecayTick { .. } => 1 << 5,
            Self::SnapshotCreated { .. } => 1 << 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_opcode() {
        let req = Request::Ping;
        assert_eq!(req.opcode(), OpCode::SysPing);

        let req = Request::LineageCreate {
            id: "test".into(),
            energy: 1.0,
            threshold: 0.5,
            decay_rate: 0.001,
        };
        assert_eq!(req.opcode(), OpCode::LineageCreate);
    }

    #[test]
    fn test_event_mask() {
        let event = Event::LineageCreated {
            id: "x".into(),
            energy: 1.0,
        };
        assert_eq!(event.mask_bit(), 1);

        let event = Event::BondCreated {
            source: "a".into(),
            target: "b".into(),
            strength: 0.5,
        };
        assert_eq!(event.mask_bit(), 1 << 3);
    }
}
