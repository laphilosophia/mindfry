//! Decay Engine - Background energy decay computation
//!
//! Uses Rayon for parallel decay processing across lineages.
//! Includes a pre-computed lookup table (LUT) for fast decay calculation.

use rayon::prelude::*;

use crate::arena::{Lineage, PsycheArena};
use crate::graph::{BondGraph, BOND_PRUNE_THRESHOLD};

/// Decay engine configuration
#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// How often to run decay tick (milliseconds)
    pub tick_interval_ms: u64,

    /// Minimum energy before lineage is considered "dead"
    pub min_energy_threshold: f32,

    /// Bond pruning threshold
    pub bond_prune_threshold: f32,

    /// Whether to use parallel processing
    pub parallel: bool,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            tick_interval_ms: 100,
            min_energy_threshold: 0.001,
            bond_prune_threshold: BOND_PRUNE_THRESHOLD,
            parallel: true,
        }
    }
}

/// Pre-computed decay lookup table
///
/// Maps (decay_rate_bucket, time_bucket) -> decay_factor
/// This eliminates exp() calls in hot paths.
pub struct DecayLUT {
    /// LUT data: rate_buckets * time_buckets
    data: Vec<f32>,
    /// Number of rate buckets
    rate_buckets: usize,
    /// Number of time buckets
    time_buckets: usize,
    /// Time bucket boundaries (seconds)
    time_boundaries: Vec<f32>,
}

impl DecayLUT {
    /// Create a new decay LUT
    pub fn new() -> Self {
        const RATE_BUCKETS: usize = 256;
        const TIME_BUCKETS: usize = 32;

        // Logarithmic time boundaries (seconds)
        let time_boundaries: Vec<f32> = vec![
            0.0, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0, 120.0,
            300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 14400.0, 21600.0,
            43200.0, 86400.0, 172800.0, 259200.0, 432000.0, 604800.0,
            1209600.0, 2592000.0, 5184000.0, 7776000.0, 15552000.0, 31104000.0,
        ];

        let mut data = Vec::with_capacity(RATE_BUCKETS * TIME_BUCKETS);

        for r in 0..RATE_BUCKETS {
            // Logarithmic rate scale: 0 = 0, 255 = 1.0/sec
            let rate = if r == 0 {
                0.0
            } else {
                (10.0_f32).powf((r as f32 / 255.0) * 3.0 - 6.0)
            };

            for t in 0..TIME_BUCKETS {
                let elapsed = time_boundaries[t];
                let factor = (-rate * elapsed).exp();
                data.push(factor);
            }
        }

        Self {
            data,
            rate_buckets: RATE_BUCKETS,
            time_buckets: TIME_BUCKETS,
            time_boundaries,
        }
    }

    /// Get decay factor for given rate and elapsed time
    #[inline]
    pub fn get(&self, decay_rate: f32, elapsed_secs: f32) -> f32 {
        let rate_bucket = self.rate_to_bucket(decay_rate);
        let time_bucket = self.time_to_bucket(elapsed_secs);
        self.data[rate_bucket * self.time_buckets + time_bucket]
    }

    /// Convert decay rate to bucket index
    #[inline]
    fn rate_to_bucket(&self, rate: f32) -> usize {
        if rate <= 0.0 {
            return 0;
        }
        // Inverse of: rate = 10^((bucket/255) * 3 - 6)
        // bucket = (log10(rate) + 6) / 3 * 255
        let log_rate = rate.log10();
        let bucket = ((log_rate + 6.0) / 3.0 * 255.0).round() as usize;
        bucket.min(self.rate_buckets - 1)
    }

    /// Convert elapsed time to bucket index
    #[inline]
    fn time_to_bucket(&self, elapsed: f32) -> usize {
        for (i, &boundary) in self.time_boundaries.iter().enumerate().rev() {
            if elapsed >= boundary {
                return i;
            }
        }
        0
    }
}

impl Default for DecayLUT {
    fn default() -> Self {
        Self::new()
    }
}

/// Background decay engine
pub struct DecayEngine {
    /// Configuration
    config: DecayConfig,
    /// Pre-computed decay LUT
    lut: DecayLUT,
    /// Last tick timestamp (nanoseconds)
    last_tick: u64,
}

impl DecayEngine {
    /// Create a new decay engine
    pub fn new(config: DecayConfig) -> Self {
        Self {
            config,
            lut: DecayLUT::new(),
            last_tick: now_nanos(),
        }
    }

    /// Get the decay factor using LUT
    #[inline]
    pub fn decay_factor(&self, decay_rate: f32, elapsed_secs: f32) -> f32 {
        self.lut.get(decay_rate, elapsed_secs)
    }

    /// Run a decay tick on the psyche arena
    ///
    /// Returns the number of lineages that dropped below threshold
    pub fn tick_psyche(&mut self, psyche: &mut PsycheArena) -> DecayTickResult {
        let now = now_nanos();

        let mut dead_count = 0;
        let mut processed = 0;

        // Get mutable slice for parallel processing
        // Note: We iterate and check, but don't actually mutate energy here
        // because decay is lazy (computed on access). This tick is for cleanup.
        for (_id, lineage) in psyche.iter_mut() {
            processed += 1;
            let energy = lineage.current_energy();
            if energy < self.config.min_energy_threshold {
                dead_count += 1;
            }
        }

        self.last_tick = now;

        DecayTickResult {
            processed,
            dead_count,
            elapsed_ms: (now - self.last_tick) / 1_000_000,
        }
    }

    /// Run bond pruning
    ///
    /// Returns the number of bonds pruned
    pub fn prune_bonds(&self, bonds: &mut BondGraph) -> usize {
        bonds.prune(self.config.bond_prune_threshold)
    }

    /// Batch compute decay factors (for parallel processing)
    pub fn batch_decay_factors(
        &self,
        lineages: &[Lineage],
    ) -> Vec<f32> {
        if self.config.parallel {
            lineages
                .par_iter()
                .map(|l| l.current_energy())
                .collect()
        } else {
            lineages
                .iter()
                .map(|l| l.current_energy())
                .collect()
        }
    }
}

impl Default for DecayEngine {
    fn default() -> Self {
        Self::new(DecayConfig::default())
    }
}

/// Result of a decay tick
#[derive(Debug, Clone, Copy)]
pub struct DecayTickResult {
    /// Number of lineages processed
    pub processed: usize,
    /// Number of lineages below death threshold
    pub dead_count: usize,
    /// Elapsed time since last tick (ms)
    pub elapsed_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// TIME UTILITIES
// ═══════════════════════════════════════════════════════════════

#[inline]
fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_lut_creation() {
        let lut = DecayLUT::new();
        assert!(!lut.data.is_empty());
    }

    #[test]
    fn test_decay_lut_zero_rate() {
        let lut = DecayLUT::new();
        // Zero decay rate should always return 1.0
        assert_eq!(lut.get(0.0, 0.0), 1.0);
        assert_eq!(lut.get(0.0, 3600.0), 1.0);
    }

    #[test]
    fn test_decay_lut_fast_decay() {
        let lut = DecayLUT::new();
        // Fast decay should approach 0 over time
        let factor_1s = lut.get(0.5, 1.0);
        let factor_10s = lut.get(0.5, 10.0);
        assert!(factor_10s < factor_1s);
    }

    #[test]
    fn test_decay_engine_creation() {
        let engine = DecayEngine::default();
        assert!(engine.decay_factor(0.001, 0.0) > 0.99);
    }

    #[test]
    fn test_decay_tick() {
        let mut engine = DecayEngine::default();
        let mut psyche = PsycheArena::with_capacity(100);

        psyche.alloc(Lineage::new(0.5));
        psyche.alloc(Lineage::new(0.003)); // Below threshold

        let result = engine.tick_psyche(&mut psyche);
        assert_eq!(result.processed, 2);
    }
}
