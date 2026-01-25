//! Crash Recovery — Trauma Detection and Resistance Building
//!
//! Detects unclean shutdowns (shock) and prolonged inactivity (coma),
//! then builds temporary resistance that decays over time.

use serde::{Deserialize, Serialize};

/// Coma threshold: 1 hour of downtime
pub const COMA_THRESHOLD_SECS: u64 = 3600;

/// Resistance decay rate per tick
pub const RESISTANCE_DECAY_RATE: f32 = 0.01;

/// Recovery state after restart
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    /// Clean restart, no trauma
    Normal,
    /// Unclean shutdown detected (no graceful marker)
    Shock,
    /// Prolonged downtime detected
    Coma,
}

impl RecoveryState {
    /// Get trauma intensity for resistance building
    pub fn intensity(&self) -> f32 {
        match self {
            RecoveryState::Normal => 0.0,
            RecoveryState::Shock => 0.3, // Moderate trauma
            RecoveryState::Coma => 0.5,  // Deeper trauma
        }
    }

    /// Get description for logging
    pub fn description(&self) -> &'static str {
        match self {
            RecoveryState::Normal => "clean restart",
            RecoveryState::Shock => "shock (unclean shutdown)",
            RecoveryState::Coma => "coma (prolonged inactivity)",
        }
    }
}

/// Shutdown marker stored in persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownMarker {
    /// Timestamp of shutdown (Unix seconds)
    pub timestamp: u64,
    /// Was this a graceful shutdown?
    pub graceful: bool,
    /// Version that created this marker
    pub version: String,
}

impl ShutdownMarker {
    /// Create a new graceful shutdown marker
    pub fn graceful() -> Self {
        Self {
            timestamp: now_secs(),
            graceful: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Recovery analyzer for detecting trauma on startup
pub struct RecoveryAnalyzer {
    /// Last shutdown marker (if found)
    last_marker: Option<ShutdownMarker>,
    /// Current startup timestamp
    startup_time: u64,
}

impl RecoveryAnalyzer {
    /// Create analyzer from stored marker
    pub fn new(last_marker: Option<ShutdownMarker>) -> Self {
        Self {
            last_marker,
            startup_time: now_secs(),
        }
    }

    /// Analyze restart conditions
    pub fn analyze(&self) -> RecoveryState {
        match &self.last_marker {
            None => {
                // No marker at all — first run or complete data loss
                // Treat as normal (genesis mode)
                RecoveryState::Normal
            }
            Some(marker) if !marker.graceful => {
                // Marker exists but not graceful — shock
                RecoveryState::Shock
            }
            Some(marker) => {
                // Graceful shutdown — check for coma
                let downtime = self.startup_time.saturating_sub(marker.timestamp);
                if downtime > COMA_THRESHOLD_SECS {
                    RecoveryState::Coma
                } else {
                    RecoveryState::Normal
                }
            }
        }
    }

    /// Get downtime duration in seconds
    pub fn downtime_secs(&self) -> u64 {
        self.last_marker
            .as_ref()
            .map(|m| self.startup_time.saturating_sub(m.timestamp))
            .unwrap_or(0)
    }
}

/// Decay resistance over time
/// Returns new resistance value after decay
#[inline]
pub fn decay_resistance(current: f32, rate: f32) -> f32 {
    (current - rate).max(0.0)
}

#[inline]
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_state_intensity() {
        assert_eq!(RecoveryState::Normal.intensity(), 0.0);
        assert!(RecoveryState::Shock.intensity() > 0.0);
        assert!(RecoveryState::Coma.intensity() > RecoveryState::Shock.intensity());
    }

    #[test]
    fn test_analyzer_no_marker() {
        let analyzer = RecoveryAnalyzer::new(None);
        assert_eq!(analyzer.analyze(), RecoveryState::Normal);
    }

    #[test]
    fn test_analyzer_graceful_marker() {
        let marker = ShutdownMarker {
            timestamp: now_secs() - 10, // 10 seconds ago
            graceful: true,
            version: "test".into(),
        };
        let analyzer = RecoveryAnalyzer::new(Some(marker));
        assert_eq!(analyzer.analyze(), RecoveryState::Normal);
    }

    #[test]
    fn test_analyzer_shock() {
        let marker = ShutdownMarker {
            timestamp: now_secs() - 10,
            graceful: false, // Not graceful!
            version: "test".into(),
        };
        let analyzer = RecoveryAnalyzer::new(Some(marker));
        assert_eq!(analyzer.analyze(), RecoveryState::Shock);
    }

    #[test]
    fn test_analyzer_coma() {
        let marker = ShutdownMarker {
            timestamp: now_secs() - COMA_THRESHOLD_SECS - 100, // Over threshold
            graceful: true,
            version: "test".into(),
        };
        let analyzer = RecoveryAnalyzer::new(Some(marker));
        assert_eq!(analyzer.analyze(), RecoveryState::Coma);
    }

    #[test]
    fn test_resistance_decay() {
        let resistance = 0.5;
        let decayed = decay_resistance(resistance, RESISTANCE_DECAY_RATE);
        assert!(decayed < resistance);
        assert!(decayed > 0.0);

        // Decay to zero
        let zero = decay_resistance(0.005, RESISTANCE_DECAY_RATE);
        assert_eq!(zero, 0.0);
    }
}
