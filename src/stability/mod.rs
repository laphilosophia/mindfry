//! Stability Layer - MindFry's Resilience Infrastructure
//!
//! The cognitive substrate's "immune system" that handles:
//! - Self-diagnostic health monitoring
//! - Exhaustion state (circuit breaker)
//! - Shutdown experience recording
//! - Crash recovery (shock/coma states)
//!
//! ## System Lineages
//!
//! Reserved namespace `_system.*` for stability-related memories:
//! - `_system.health` - Self-diagnostic pulse
//! - `_system.state` - Current exhaustion level
//! - `_system.shutdown.*` - Shutdown experience records
//! - `_system.shock` - Crash trauma
//! - `_system.coma` - Prolonged inactivity
//! - `_system.resistance` - Built from past challenges

pub mod exhaustion;
pub mod health;
pub mod recovery;
pub mod shutdown;
pub mod warmup;

pub use exhaustion::{
    ExhaustionLevel, ExhaustionMonitor, ExhaustionThresholds, ExhaustionTuner, TunerConfig,
    TunerStats,
};
pub use health::{HealthStatus, SelfDiagnostic};
pub use recovery::{
    RecoveryAnalyzer, RecoveryState, ShutdownMarker, COMA_THRESHOLD_SECS, RESISTANCE_DECAY_RATE,
};
pub use shutdown::{ShutdownReason, ShutdownTracker};
pub use warmup::{WarmupState, WarmupTracker};

/// System lineage key constants
pub mod lineages {
    /// Self-diagnostic health lineage
    pub const HEALTH: &str = "_system.health";
    /// Current exhaustion level
    pub const STATE: &str = "_system.state";
    /// Graceful shutdown record
    pub const SHUTDOWN_GRACEFUL: &str = "_system.shutdown.graceful";
    /// Forced shutdown record (signal)
    pub const SHUTDOWN_FORCED: &str = "_system.shutdown.forced";
    /// Crash trauma
    pub const SHOCK: &str = "_system.shock";
    /// Prolonged inactivity
    pub const COMA: &str = "_system.coma";
    /// Restart trauma
    pub const INSTABILITY: &str = "_system.instability";
    /// Resistance built from challenges
    pub const RESISTANCE: &str = "_system.resistance";
}
