//! Decay Engine - Background energy decay computation
//!
//! Uses Rayon for parallel decay processing across lineages.
//! Includes a pre-computed lookup table (LUT) for fast decay calculation.

use rayon::prelude::*;

use crate::arena::{Lineage, PsycheArena};
use crate::graph::{BOND_PRUNE_THRESHOLD, BondGraph};

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
            0.0, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0,
            1800.0, 3600.0, 7200.0, 14400.0, 21600.0, 43200.0, 86400.0, 172800.0, 259200.0,
            432000.0, 604800.0, 1209600.0, 2592000.0, 5184000.0, 7776000.0, 15552000.0, 31104000.0,
        ];

        let mut data = Vec::with_capacity(RATE_BUCKETS * TIME_BUCKETS);

        for r in 0..RATE_BUCKETS {
            // Logarithmic rate scale: 0 = 0, 255 = 1.0/sec
            let rate = if r == 0 {
                0.0
            } else {
                (10.0_f32).powf((r as f32 / 255.0) * 3.0 - 6.0)
            };

            for &elapsed in time_boundaries.iter().take(TIME_BUCKETS) {
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
    pub fn batch_decay_factors(&self, lineages: &[Lineage]) -> Vec<f32> {
        if self.config.parallel {
            lineages.par_iter().map(|l| l.current_energy()).collect()
        } else {
            lineages.iter().map(|l| l.current_energy()).collect()
        }
    }

    /// Process garbage collection with Cortex ternary logic and RetentionBuffer.
    ///
    /// This method evaluates each lineage's viability using the Cortex's
    /// personality-aware decision making, and uses the RetentionBuffer
    /// for TTL-based safe deletion.
    ///
    /// # Returns
    /// `GcResult` with counts of processed, retained, pending, and pruned lineages.
    pub fn process_gc(
        &self,
        psyche: &mut PsycheArena,
        cortex: &mut crate::setun::Cortex,
    ) -> GcResult {
        use crate::setun::{Trit, dimension};

        let mut processed = 0;
        let mut retained = 0; // Healthy, stay alive
        let mut pending = 0; // In retention buffer
        let mut disposables = Vec::new(); // Ready for deletion

        // Get preservation bias from personality
        let preservation_bias =
            cortex.personality().get(dimension::PRESERVATION).weight() as f64 * 0.1;

        for (id, lineage) in psyche.iter_mut() {
            processed += 1;
            let energy = lineage.current_energy() as f64;

            // Viability score: energy relative to death threshold
            // Positive = healthy, negative = dying
            let viability_score = energy - self.config.min_energy_threshold as f64;

            // Apply preservation bias to the decision
            let adjusted_score = viability_score + preservation_bias;

            let decision = cortex.decide(adjusted_score);

            match decision {
                Trit::True => {
                    // STABLE: Lineage is healthy
                    // Remove from retention buffer if present
                    cortex.retention_mut().restore(id.index());
                    retained += 1;
                }
                Trit::Unknown => {
                    // UNSTABLE: Lineage is borderline
                    // Add to retention buffer or tick TTL
                    if cortex.retention_mut().mark_or_tick(id.index()) {
                        disposables.push(id);
                    } else {
                        pending += 1;
                    }
                }
                Trit::False => {
                    // OBSOLETE: Lineage is dying
                    // Fast-track through retention buffer
                    if cortex.retention_mut().mark_or_tick(id.index()) {
                        disposables.push(id);
                    } else {
                        pending += 1;
                    }
                }
            }
        }

        // Execute pruning
        let pruned = disposables.len();
        for id in disposables {
            psyche.free(id);
        }

        GcResult {
            processed,
            retained,
            pending,
            pruned,
        }
    }
}

/// Result of a garbage collection pass
#[derive(Debug, Clone, Default)]
pub struct GcResult {
    /// Total lineages processed
    pub processed: usize,
    /// Lineages that are healthy (Trit::True)
    pub retained: usize,
    /// Lineages in retention buffer (pending deletion)
    pub pending: usize,
    /// Lineages pruned this tick
    pub pruned: usize,
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

    #[test]
    fn test_process_gc_with_retention() {
        use crate::setun::{Cortex, Octet, Trit, dimension};

        let engine = DecayEngine::default();
        let mut psyche = PsycheArena::with_capacity(100);

        // Create personality with high preservation
        let mut personality = Octet::neutral();
        personality.set(dimension::PRESERVATION, Trit::True);
        let mut cortex = Cortex::new(personality);

        // Create lineages with varying energy levels
        psyche.alloc(Lineage::new(0.9)); // Healthy - should be retained
        psyche.alloc(Lineage::new(0.05)); // Dying - will enter buffer
        psyche.alloc(Lineage::new(0.03)); // Dying - will enter buffer

        // GC Pass 1: Dying lineages enter retention buffer
        let result1 = engine.process_gc(&mut psyche, &mut cortex);
        assert_eq!(result1.processed, 3);
        assert_eq!(result1.retained, 1); // 0.9 energy one
        assert_eq!(result1.pending, 2); // Two dying ones in buffer
        assert_eq!(result1.pruned, 0); // Nothing pruned yet
        assert_eq!(cortex.pending_removal_count(), 2);

        // GC Pass 2: TTL ticks down
        let result2 = engine.process_gc(&mut psyche, &mut cortex);
        assert_eq!(result2.pruned, 0); // Still in buffer
        assert_eq!(result2.pending, 2);

        // GC Pass 3: TTL ticks down more
        let result3 = engine.process_gc(&mut psyche, &mut cortex);
        assert_eq!(result3.pruned, 0); // Still in buffer

        // GC Pass 4: TTL expires, lineages are pruned
        let result4 = engine.process_gc(&mut psyche, &mut cortex);
        assert_eq!(result4.pruned, 2); // Both dying lineages pruned
        assert_eq!(psyche.len(), 1); // Only healthy one remains
        assert_eq!(cortex.pending_removal_count(), 0);
    }
}
