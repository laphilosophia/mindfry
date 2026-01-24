//! Warmup Tracker - Progressive availability during resurrection
//!
//! Enables zero startup delay by allowing the server to accept connections
//! while snapshot loading happens in the background.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Server warmup state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WarmupState {
    /// No snapshot found, server is ready immediately
    Cold = 0,
    /// Snapshot loading in progress
    Resurrecting = 1,
    /// Fully operational
    Ready = 2,
}

impl From<u8> for WarmupState {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Cold,
            1 => Self::Resurrecting,
            2 => Self::Ready,
            _ => Self::Cold,
        }
    }
}

/// Thread-safe warmup state tracker
#[derive(Clone)]
pub struct WarmupTracker {
    state: Arc<AtomicU8>,
}

impl WarmupTracker {
    /// Create new tracker in Cold state
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(WarmupState::Cold as u8)),
        }
    }

    /// Get current state
    pub fn state(&self) -> WarmupState {
        WarmupState::from(self.state.load(Ordering::SeqCst))
    }

    /// Check if server is ready to handle requests
    pub fn is_ready(&self) -> bool {
        matches!(self.state(), WarmupState::Cold | WarmupState::Ready)
    }

    /// Transition to Resurrecting state
    pub fn begin_resurrection(&self) {
        self.state
            .store(WarmupState::Resurrecting as u8, Ordering::SeqCst);
    }

    /// Transition to Ready state
    pub fn mark_ready(&self) {
        self.state.store(WarmupState::Ready as u8, Ordering::SeqCst);
    }

    /// Stay in Cold state (no resurrection needed)
    pub fn mark_cold(&self) {
        self.state.store(WarmupState::Cold as u8, Ordering::SeqCst);
    }
}

impl Default for WarmupTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_state_transitions() {
        let tracker = WarmupTracker::new();
        assert_eq!(tracker.state(), WarmupState::Cold);
        assert!(tracker.is_ready());

        tracker.begin_resurrection();
        assert_eq!(tracker.state(), WarmupState::Resurrecting);
        assert!(!tracker.is_ready());

        tracker.mark_ready();
        assert_eq!(tracker.state(), WarmupState::Ready);
        assert!(tracker.is_ready());
    }
}
