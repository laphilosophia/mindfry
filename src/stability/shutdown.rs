//! Shutdown Experience Recording
//!
//! Every shutdown is recorded as a memory experience.
//! Intensity varies by shutdown type, affecting decay rate.

use serde::{Deserialize, Serialize};

/// Reason for shutdown - each type has different intensity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownReason {
    /// Normal, clean exit - neutral memory (fast decay)
    Graceful,
    /// Signal-based exit (SIGTERM, SIGINT) - mild scar
    Signal {
        /// Signal number that caused the exit
        signal: i32,
    },
    /// Crash - trauma (slow decay)
    Crash {
        /// Error message describing the crash
        message: String,
    },
    /// Entering coma state - prolonged inactivity
    Coma {
        /// Duration of downtime in seconds
        downtime_seconds: u64,
    },
}

impl ShutdownReason {
    /// Get intensity level (affects decay rate)
    /// Lower = faster decay, Higher = slower decay (scar)
    pub fn intensity(&self) -> f32 {
        match self {
            ShutdownReason::Graceful => 0.1,      // Fast decay - forgets easily
            ShutdownReason::Signal { .. } => 0.4, // Medium - remembers for a while
            ShutdownReason::Crash { .. } => 0.8,  // Slow - scar remains
            ShutdownReason::Coma { .. } => 0.9,   // Very slow - deep memory
        }
    }

    /// Get the target lineage key for this shutdown type
    pub fn lineage_key(&self) -> &'static str {
        match self {
            ShutdownReason::Graceful => super::lineages::SHUTDOWN_GRACEFUL,
            ShutdownReason::Signal { .. } => super::lineages::SHUTDOWN_FORCED,
            ShutdownReason::Crash { .. } => super::lineages::SHOCK,
            ShutdownReason::Coma { .. } => super::lineages::COMA,
        }
    }

    /// Get description for logging
    pub fn description(&self) -> String {
        match self {
            ShutdownReason::Graceful => "graceful shutdown".into(),
            ShutdownReason::Signal { signal } => format!("forced (signal {})", signal),
            ShutdownReason::Crash { message } => format!("crash: {}", message),
            ShutdownReason::Coma { downtime_seconds } => {
                format!("coma: {}s downtime", downtime_seconds)
            }
        }
    }
}

/// Tracks shutdown experiences
pub struct ShutdownTracker {
    /// Last shutdown reason (if any)
    pub last_reason: Option<ShutdownReason>,
    /// Timestamp of last shutdown
    pub last_timestamp: Option<u64>,
}

impl Default for ShutdownTracker {
    fn default() -> Self {
        Self {
            last_reason: None,
            last_timestamp: None,
        }
    }
}

impl ShutdownTracker {
    /// Record a shutdown event
    pub fn record(&mut self, reason: ShutdownReason, timestamp: u64) {
        self.last_reason = Some(reason);
        self.last_timestamp = Some(timestamp);
    }

    /// Check if last shutdown was traumatic
    pub fn was_traumatic(&self) -> bool {
        self.last_reason
            .as_ref()
            .map(|r| r.intensity() > 0.5)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_intensity() {
        assert!(ShutdownReason::Graceful.intensity() < 0.2);
        assert!(
            ShutdownReason::Crash {
                message: "test".into()
            }
            .intensity()
                > 0.7
        );
    }

    #[test]
    fn test_traumatic_detection() {
        let mut tracker = ShutdownTracker::default();

        tracker.record(ShutdownReason::Graceful, 1000);
        assert!(!tracker.was_traumatic());

        tracker.record(
            ShutdownReason::Crash {
                message: "panic".into(),
            },
            2000,
        );
        assert!(tracker.was_traumatic());
    }
}
