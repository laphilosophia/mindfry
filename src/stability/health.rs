//! Self-Diagnostic Health System
//!
//! MindFry's health is tracked via `_system.health` lineage.
//! Periodic self-stimulation keeps the health lineage alive.
//! If energy drops, the system is unhealthy.

use serde::{Deserialize, Serialize};

/// Health status derived from self-diagnostic lineage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is fully functional
    Healthy,
    /// System is operational but degraded
    Degraded,
    /// System needs attention
    Unhealthy,
}

impl HealthStatus {
    /// Derive status from energy level
    pub fn from_energy(energy: f32) -> Self {
        match energy {
            e if e > 0.7 => HealthStatus::Healthy,
            e if e > 0.3 => HealthStatus::Degraded,
            _ => HealthStatus::Unhealthy,
        }
    }
}

/// Self-diagnostic system using lineage-based health tracking
pub struct SelfDiagnostic {
    /// Pulse amount to add on each check
    pub pulse_amount: f32,
    /// Interval between pulses (in ticks)
    pub pulse_interval: u64,
    /// Counter for pulse timing
    tick_counter: u64,
}

impl Default for SelfDiagnostic {
    fn default() -> Self {
        Self {
            pulse_amount: 0.1,
            pulse_interval: 100, // Every 100 ticks
            tick_counter: 0,
        }
    }
}

impl SelfDiagnostic {
    /// Create with custom pulse configuration
    pub fn new(pulse_amount: f32, pulse_interval: u64) -> Self {
        Self {
            pulse_amount,
            pulse_interval,
            tick_counter: 0,
        }
    }

    /// Check if it's time for a pulse
    pub fn should_pulse(&mut self) -> bool {
        self.tick_counter += 1;
        if self.tick_counter >= self.pulse_interval {
            self.tick_counter = 0;
            true
        } else {
            false
        }
    }

    /// Get pulse amount for health lineage
    pub fn pulse_delta(&self) -> f32 {
        self.pulse_amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::from_energy(1.0), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from_energy(0.5), HealthStatus::Degraded);
        assert_eq!(HealthStatus::from_energy(0.1), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_pulse_timing() {
        let mut diag = SelfDiagnostic::new(0.1, 3);
        assert!(!diag.should_pulse()); // 1
        assert!(!diag.should_pulse()); // 2
        assert!(diag.should_pulse()); // 3 - fires
        assert!(!diag.should_pulse()); // 1 - reset
    }
}
