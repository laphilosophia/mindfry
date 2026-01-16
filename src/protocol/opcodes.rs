//! MFBP OpCode Definitions
//!
//! Each command has a unique 1-byte OpCode for fast parsing.

/// OpCode for MFBP commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    // ═══════════════════════════════════════════════════════════════
    // LINEAGE OPERATIONS (0x10-0x1F)
    // ═══════════════════════════════════════════════════════════════

    /// Create a new lineage
    /// Payload: [id_len: u16, id_bytes: [u8], energy: f32, threshold: f32, decay_rate: f32]
    LineageCreate = 0x10,

    /// Get lineage details
    /// Payload: [id_len: u16, id_bytes: [u8]]
    LineageGet = 0x11,

    /// Stimulate a lineage (inject energy)
    /// Payload: [id_len: u16, id_bytes: [u8], delta: f32]
    LineageStimulate = 0x12,

    /// Soft-delete a lineage (forget)
    /// Payload: [id_len: u16, id_bytes: [u8]]
    LineageForget = 0x13,

    /// Touch lineage (update last access)
    /// Payload: [id_len: u16, id_bytes: [u8]]
    LineageTouch = 0x14,

    // ═══════════════════════════════════════════════════════════════
    // BOND OPERATIONS (0x20-0x2F)
    // ═══════════════════════════════════════════════════════════════

    /// Create a bond between two lineages
    /// Payload: [src_len: u16, src: [u8], tgt_len: u16, tgt: [u8], strength: f32]
    BondConnect = 0x20,

    /// Reinforce an existing bond
    /// Payload: [src_len: u16, src: [u8], tgt_len: u16, tgt: [u8], delta: f32]
    BondReinforce = 0x21,

    /// Sever (disconnect) a bond
    /// Payload: [src_len: u16, src: [u8], tgt_len: u16, tgt: [u8]]
    BondSever = 0x22,

    /// Get neighbors of a lineage
    /// Payload: [id_len: u16, id_bytes: [u8]]
    BondNeighbors = 0x23,

    // ═══════════════════════════════════════════════════════════════
    // QUERY OPERATIONS (0x30-0x3F)
    // ═══════════════════════════════════════════════════════════════

    /// Query conscious lineages (energy >= threshold)
    /// Payload: [min_energy: f32]
    QueryConscious = 0x30,

    /// Get top K lineages by energy
    /// Payload: [k: u32]
    QueryTopK = 0x31,

    /// Query traumatized lineages (high rigidity)
    /// Payload: [min_rigidity: f32]
    QueryTrauma = 0x32,

    /// Query lineages by pattern
    /// Payload: [pattern_len: u16, pattern: [u8]]
    QueryPattern = 0x33,

    // ═══════════════════════════════════════════════════════════════
    // SYSTEM OPERATIONS (0x40-0x4F)
    // ═══════════════════════════════════════════════════════════════

    /// Ping (keep-alive)
    /// Payload: []
    SysPing = 0x40,

    /// Get database statistics
    /// Payload: []
    SysStats = 0x41,

    /// Create a snapshot (checkpoint)
    /// Payload: [name_len: u16, name: [u8]]
    SysSnapshot = 0x42,

    /// Restore from snapshot
    /// Payload: [name_len: u16, name: [u8]]
    SysRestore = 0x43,

    /// Freeze/Thaw decay engine
    /// Payload: [state: u8] (0 = thaw, 1 = freeze)
    SysFreeze = 0x44,

    /// Tune physics constants
    /// Payload: [param_id: u8, value: f32]
    PhysicsTune = 0x45,

    // ═══════════════════════════════════════════════════════════════
    // STREAM OPERATIONS (0x50-0x5F)
    // ═══════════════════════════════════════════════════════════════

    /// Subscribe to events
    /// Payload: [events_mask: u32]
    StreamSubscribe = 0x50,

    /// Unsubscribe from events
    /// Payload: []
    StreamUnsubscribe = 0x51,

    // ═══════════════════════════════════════════════════════════════
    // RESPONSE CODES (0xF0-0xFF)
    // ═══════════════════════════════════════════════════════════════

    /// Success response
    ResponseOk = 0xF0,

    /// Error response
    ResponseError = 0xF1,

    /// Event notification (from subscription)
    ResponseEvent = 0xF2,
}

impl OpCode {
    /// Try to parse an OpCode from a byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            // Lineage
            0x10 => Some(Self::LineageCreate),
            0x11 => Some(Self::LineageGet),
            0x12 => Some(Self::LineageStimulate),
            0x13 => Some(Self::LineageForget),
            0x14 => Some(Self::LineageTouch),
            // Bond
            0x20 => Some(Self::BondConnect),
            0x21 => Some(Self::BondReinforce),
            0x22 => Some(Self::BondSever),
            0x23 => Some(Self::BondNeighbors),
            // Query
            0x30 => Some(Self::QueryConscious),
            0x31 => Some(Self::QueryTopK),
            0x32 => Some(Self::QueryTrauma),
            0x33 => Some(Self::QueryPattern),
            // System
            0x40 => Some(Self::SysPing),
            0x41 => Some(Self::SysStats),
            0x42 => Some(Self::SysSnapshot),
            0x43 => Some(Self::SysRestore),
            0x44 => Some(Self::SysFreeze),
            0x45 => Some(Self::PhysicsTune),
            // Stream
            0x50 => Some(Self::StreamSubscribe),
            0x51 => Some(Self::StreamUnsubscribe),
            // Response
            0xF0 => Some(Self::ResponseOk),
            0xF1 => Some(Self::ResponseError),
            0xF2 => Some(Self::ResponseEvent),
            _ => None,
        }
    }

    /// Get the byte value of this OpCode
    #[inline]
    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Check if this is a response OpCode
    #[inline]
    pub fn is_response(self) -> bool {
        (self as u8) >= 0xF0
    }
}

/// Event types for subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum EventMask {
    /// Lineage created
    LineageCreated = 1 << 0,
    /// Lineage stimulated
    LineageStimulated = 1 << 1,
    /// Lineage forgotten
    LineageForgotten = 1 << 2,
    /// Bond created
    BondCreated = 1 << 3,
    /// Bond severed
    BondSevered = 1 << 4,
    /// Decay tick completed
    DecayTick = 1 << 5,
    /// Snapshot created
    SnapshotCreated = 1 << 6,
    /// All events
    All = 0xFFFFFFFF,
}

/// Physics parameters that can be tuned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicsParam {
    /// Global decay rate multiplier
    DecayMultiplier = 0x01,
    /// Trauma rigidity threshold
    TraumaThreshold = 0x02,
    /// Bond pruning threshold
    BondPruneThreshold = 0x03,
    /// Minimum energy threshold
    MinEnergyThreshold = 0x04,
}

impl PhysicsParam {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::DecayMultiplier),
            0x02 => Some(Self::TraumaThreshold),
            0x03 => Some(Self::BondPruneThreshold),
            0x04 => Some(Self::MinEnergyThreshold),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        let op = OpCode::LineageCreate;
        assert_eq!(OpCode::from_byte(op.as_byte()), Some(op));
    }

    #[test]
    fn test_opcode_ranges() {
        assert!(OpCode::LineageCreate.as_byte() >= 0x10);
        assert!(OpCode::LineageCreate.as_byte() < 0x20);

        assert!(OpCode::BondConnect.as_byte() >= 0x20);
        assert!(OpCode::BondConnect.as_byte() < 0x30);

        assert!(OpCode::ResponseOk.is_response());
        assert!(!OpCode::SysPing.is_response());
    }
}
