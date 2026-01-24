//! Exhaustion State - MindFry's Circuit Breaker
//!
//! Instead of a mechanical circuit breaker, MindFry uses an
//! exhaustion model based on the `_system.state` lineage.
//!
//! When exhausted, MindFry gracefully degrades instead of failing.

use serde::{Deserialize, Serialize};

/// Configuration for exhaustion level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExhaustionThresholds {
    /// Above this = Normal
    pub normal: f32,
    /// Above this = Elevated
    pub elevated: f32,
    /// Above this = Exhausted (below = Emergency)
    pub exhausted: f32,
}

impl Default for ExhaustionThresholds {
    fn default() -> Self {
        Self {
            normal: 0.7,
            elevated: 0.4,
            exhausted: 0.1,
        }
    }
}

/// Exhaustion levels for the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExhaustionLevel {
    /// Full capacity, all operations allowed
    Normal,
    /// Slight pressure, still accepting but monitoring
    Elevated,
    /// Rejecting new writes, reads OK
    Exhausted,
    /// Read-only, recovery mode
    Emergency,
}

impl ExhaustionLevel {
    /// Get level from energy value using default thresholds
    pub fn from_energy(energy: f32) -> Self {
        Self::from_energy_with_thresholds(energy, &ExhaustionThresholds::default())
    }

    /// Get level from energy value with custom thresholds
    pub fn from_energy_with_thresholds(energy: f32, thresholds: &ExhaustionThresholds) -> Self {
        match energy {
            e if e > thresholds.normal => ExhaustionLevel::Normal,
            e if e > thresholds.elevated => ExhaustionLevel::Elevated,
            e if e > thresholds.exhausted => ExhaustionLevel::Exhausted,
            _ => ExhaustionLevel::Emergency,
        }
    }

    /// Check if writes are allowed
    pub fn allows_writes(&self) -> bool {
        matches!(self, ExhaustionLevel::Normal | ExhaustionLevel::Elevated)
    }

    /// Check if any operations are allowed
    pub fn allows_operations(&self) -> bool {
        !matches!(self, ExhaustionLevel::Emergency)
    }
}

/// Monitors and manages system exhaustion state
pub struct ExhaustionMonitor {
    /// Recovery rate per tick (how fast energy restores)
    pub recovery_rate: f32,
    /// Cost per operation
    pub operation_cost: f32,
}

impl Default for ExhaustionMonitor {
    fn default() -> Self {
        Self {
            recovery_rate: 0.05,
            operation_cost: 0.001,
        }
    }
}

impl ExhaustionMonitor {
    /// Create new monitor with custom rates
    pub fn new(recovery_rate: f32, operation_cost: f32) -> Self {
        Self {
            recovery_rate,
            operation_cost,
        }
    }

    /// Calculate work cost based on operation type
    pub fn calculate_cost(&self, is_write: bool, propagation_depth: usize) -> f32 {
        let base = self.operation_cost;
        let write_multiplier = if is_write { 2.0 } else { 1.0 };
        let depth_multiplier = 1.0 + (propagation_depth as f32 * 0.1);
        base * write_multiplier * depth_multiplier
    }
}

// ═══════════════════════════════════════════════════════════════
// AUTOTUNER (Ported from Atrion RFC-0008)
// ═══════════════════════════════════════════════════════════════

/// Configuration for ExhaustionTuner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunerConfig {
    /// Rolling window size for EMA calculation
    pub window_size: usize,
    /// Sensitivity multiplier - k in (μ + kσ)
    pub sensitivity: f32,
    /// Minimum threshold floor (safety net)
    pub min_floor: f32,
    /// Maximum threshold ceiling
    pub hard_ceiling: f32,
    /// Ticks before adaptive mode activates
    pub warmup_ticks: usize,
}

impl Default for TunerConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            sensitivity: 3.0, // 3-sigma (99.7% normal distribution)
            min_floor: 0.1,
            hard_ceiling: 0.9,
            warmup_ticks: 50,
        }
    }
}

/// Exponential Moving Average Accumulator
///
/// O(1) memory complexity with decay-weighted recent bias.
/// Ideal for dynamic patterns.
struct EMAAccumulator {
    mean: f32,
    variance: f32,
    count: usize,
    alpha: f32,
}

impl EMAAccumulator {
    fn new(window_size: usize) -> Self {
        // α = 2 / (N + 1) — standard EMA smoothing factor
        let alpha = 2.0 / (window_size as f32 + 1.0);
        Self {
            mean: 0.0,
            variance: 0.0,
            count: 0,
            alpha,
        }
    }

    fn update(&mut self, x: f32) {
        self.count += 1;

        if self.count == 1 {
            self.mean = x;
            self.variance = 0.0;
            return;
        }

        let delta = x - self.mean;
        self.mean += self.alpha * delta;

        // Exponential variance estimation
        self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * delta * delta);
    }

    fn get_mean(&self) -> f32 {
        self.mean
    }

    fn get_std_dev(&self) -> f32 {
        self.variance.max(0.0).sqrt()
    }

    fn get_count(&self) -> usize {
        self.count
    }

    fn reset(&mut self) {
        self.mean = 0.0;
        self.variance = 0.0;
        self.count = 0;
    }
}

/// Tuner statistics snapshot
#[derive(Debug, Clone)]
pub struct TunerStats {
    /// Current mean
    pub mean: f32,
    /// Current standard deviation
    pub std_dev: f32,
    /// Sample count
    pub sample_count: usize,
    /// Whether warmup is complete
    pub is_warmed_up: bool,
}

/// Exhaustion Tuner - Adaptive threshold calculator
///
/// Learns energy patterns and computes dynamic exhaustion thresholds
/// using statistical deviation (μ + kσ).
///
/// Ported from Atrion AutoTuner (RFC-0008).
pub struct ExhaustionTuner {
    config: TunerConfig,
    accumulator: EMAAccumulator,
    #[allow(dead_code)] // Reserved for future warmup period behavior
    fallback_threshold: f32,
}

impl ExhaustionTuner {
    /// Create new tuner with config
    pub fn new(config: TunerConfig, fallback_threshold: f32) -> Self {
        let accumulator = EMAAccumulator::new(config.window_size);
        Self {
            config,
            accumulator,
            fallback_threshold,
        }
    }

    /// Feed an energy observation to the tuner
    pub fn observe(&mut self, energy: f32) {
        self.accumulator.update(energy);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> TunerStats {
        TunerStats {
            mean: self.accumulator.get_mean(),
            std_dev: self.accumulator.get_std_dev(),
            sample_count: self.accumulator.get_count(),
            is_warmed_up: self.accumulator.get_count() >= self.config.warmup_ticks,
        }
    }

    /// Compute dynamic exhaustion threshold
    ///
    /// Formula: clamp(μ - kσ, min_floor, hard_ceiling)
    /// Note: We subtract because lower energy = more exhausted
    pub fn compute_threshold(&self) -> ExhaustionThresholds {
        let stats = self.get_stats();

        if !stats.is_warmed_up {
            return ExhaustionThresholds::default();
        }

        // Dynamic thresholds based on learned patterns
        let normal = (stats.mean - self.config.sensitivity * 0.3 * stats.std_dev)
            .clamp(self.config.min_floor, self.config.hard_ceiling);
        let elevated = (stats.mean - self.config.sensitivity * 0.6 * stats.std_dev)
            .clamp(self.config.min_floor, normal);
        let exhausted = (stats.mean - self.config.sensitivity * stats.std_dev)
            .clamp(self.config.min_floor, elevated);

        ExhaustionThresholds {
            normal,
            elevated,
            exhausted,
        }
    }

    /// Get current exhaustion level using learned thresholds
    pub fn get_level(&self, energy: f32) -> ExhaustionLevel {
        let thresholds = self.compute_threshold();
        ExhaustionLevel::from_energy_with_thresholds(energy, &thresholds)
    }

    /// Reset tuner state
    pub fn reset(&mut self) {
        self.accumulator.reset();
    }
}

impl Default for ExhaustionTuner {
    fn default() -> Self {
        Self::new(TunerConfig::default(), 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustion_levels() {
        assert_eq!(ExhaustionLevel::from_energy(1.0), ExhaustionLevel::Normal);
        assert_eq!(ExhaustionLevel::from_energy(0.5), ExhaustionLevel::Elevated);
        assert_eq!(
            ExhaustionLevel::from_energy(0.2),
            ExhaustionLevel::Exhausted
        );
        assert_eq!(
            ExhaustionLevel::from_energy(0.05),
            ExhaustionLevel::Emergency
        );
    }

    #[test]
    fn test_write_permissions() {
        assert!(ExhaustionLevel::Normal.allows_writes());
        assert!(ExhaustionLevel::Elevated.allows_writes());
        assert!(!ExhaustionLevel::Exhausted.allows_writes());
        assert!(!ExhaustionLevel::Emergency.allows_writes());
    }

    #[test]
    fn test_tuner_warmup() {
        let mut tuner = ExhaustionTuner::new(
            TunerConfig {
                warmup_ticks: 5,
                ..Default::default()
            },
            0.5,
        );

        // Before warmup
        assert!(!tuner.get_stats().is_warmed_up);

        // Feed observations
        for _ in 0..5 {
            tuner.observe(0.8);
        }

        // After warmup
        assert!(tuner.get_stats().is_warmed_up);
    }

    #[test]
    fn test_tuner_learns_pattern() {
        let mut tuner = ExhaustionTuner::new(
            TunerConfig {
                warmup_ticks: 10,
                window_size: 20,
                ..Default::default()
            },
            0.5,
        );

        // Feed stable high-energy pattern
        for _ in 0..20 {
            tuner.observe(0.85);
        }

        let stats = tuner.get_stats();
        assert!(stats.mean > 0.8);
        assert!(stats.std_dev < 0.1); // Low variance
    }
}
