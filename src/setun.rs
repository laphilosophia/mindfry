//! # Setun - Balanced Ternary Decision Engine
//!
//! A cognitive decision layer inspired by the Soviet Setun computer's balanced ternary logic.
//! Instead of binary (0, 1), uses ternary (-1, 0, +1) to model biological states:
//!
//! - **True (+1)**: Excitation (Glutamate) - Active approval
//! - **Unknown (0)**: Latent (Resting) - Insufficient data / Indecision
//! - **False (-1)**: Inhibition (GABA) - Active rejection
//!
//! ## Core Components
//!
//! - [`Trit`]: The atomic ternary digit
//! - [`Octet`]: 8-dimensional personality/event vector
//! - [`Quantizer`]: Analog-to-digital converter with mood modulation
//!
//! ## Design Rationale
//!
//! Binary systems cannot express "I don't know" naturally. Ternary logic provides:
//! 1. **Anti-Hallucination**: Unknown state prevents forced decisions
//! 2. **Active Inhibition**: -1 is resistance, not absence
//! 3. **Dialectics**: Thesis (+1) vs Antithesis (-1) yields Synthesis (0)

use std::ops::{Mul, Not};

use serde::{Deserialize, Serialize};

// ============================================================================
// 1. THE ATOM: TRIT
// ============================================================================

/// Balanced Ternary Primitive
///
/// Occupies 1 byte in memory (i8) but carries 1.58 bits of information (log2(3)).
/// This is the fundamental unit of the Setun decision engine.
///
/// # Examples
///
/// ```
/// use mindfry::setun::Trit;
///
/// // Consensus: Same-sign strengthens, opposite-sign cancels
/// assert_eq!(Trit::True * Trit::True, Trit::True);
/// assert_eq!(Trit::True * Trit::False, Trit::False);
/// assert_eq!(Trit::Unknown * Trit::True, Trit::Unknown);
///
/// // Inversion
/// assert_eq!(!Trit::True, Trit::False);
/// assert_eq!(!Trit::Unknown, Trit::Unknown);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(i8)]
pub enum Trit {
    /// Inhibition / Rejection / Negative (-1)
    /// Biological analog: GABA neurotransmitter
    False = -1,

    /// Latent / Waiting / Neutral (0)
    /// Represents insufficient data or active indecision
    #[default]
    Unknown = 0,

    /// Excitation / Approval / Positive (+1)
    /// Biological analog: Glutamate neurotransmitter
    True = 1,
}

impl Trit {
    /// Consensus Logic (Multiplication)
    ///
    /// Two parties in agreement strengthen each other (+ * + = +).
    /// Opposing parties cancel out (+ * - = -).
    /// Unknown absorbs everything (0 * x = 0).
    ///
    /// This models synaptic reinforcement in biological networks.
    #[inline]
    pub fn consensus(self, other: Trit) -> Trit {
        // SAFETY: The product of any two values in {-1, 0, 1} is always in {-1, 0, 1}
        let val = (self as i8) * (other as i8);
        // Using match instead of transmute for safety
        match val {
            -1 => Trit::False,
            0 => Trit::Unknown,
            1 => Trit::True,
            // Unreachable due to i8 * i8 constraints, but satisfies compiler
            _ => Trit::Unknown,
        }
    }

    /// Get the numeric weight of this Trit for aggregation
    #[inline]
    pub const fn weight(self) -> i8 {
        self as i8
    }

    /// Inversion (Negation)
    ///
    /// True ↔ False, Unknown stays Unknown.
    /// Models inhibitory feedback loops.
    #[inline]
    pub const fn invert(self) -> Trit {
        match self {
            Trit::True => Trit::False,
            Trit::False => Trit::True,
            Trit::Unknown => Trit::Unknown,
        }
    }

    /// Convert from i8 with saturation
    ///
    /// Values > 0 become True, < 0 become False, == 0 becomes Unknown.
    #[inline]
    pub const fn from_i8_saturating(val: i8) -> Trit {
        if val > 0 {
            Trit::True
        } else if val < 0 {
            Trit::False
        } else {
            Trit::Unknown
        }
    }
}

// Operator Overloading for ergonomic code
impl Mul for Trit {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.consensus(rhs)
    }
}

impl Not for Trit {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        self.invert()
    }
}

// ============================================================================
// 2. THE MOLECULE: OCTET (Personality Vector)
// ============================================================================

/// Semantic dimension indices for the Octet personality vector.
/// These represent the "Big 8" cognitive dimensions of a MindFry entity.
pub mod dimension {
    /// Curiosity: Drive to explore new data
    pub const CURIOSITY: usize = 0;
    /// Preservation: Resistance to deletion/decay
    pub const PRESERVATION: usize = 1;
    /// Efficiency: Resource conservation preference
    pub const EFFICIENCY: usize = 2;
    /// Empathy: Alignment with user/external signals
    pub const EMPATHY: usize = 3;
    /// Rigidity: Rule adherence vs flexibility
    pub const RIGIDITY: usize = 4;
    /// Volatility: Rate of opinion change
    pub const VOLATILITY: usize = 5;
    /// Aggression: Proactive cleanup/pruning drive
    pub const AGGRESSION: usize = 6;
    /// Latency: Patience / tolerance for waiting
    pub const LATENCY: usize = 7;
}

/// 8-Dimensional Personality/Event Vector
///
/// Stack-allocated (no heap), cache-friendly (8 bytes = single cache line fetch).
/// Represents either an entity's personality or an event's characteristics.
///
/// # Examples
///
/// ```
/// use mindfry::setun::{Octet, Trit};
///
/// let curious_aggressive = Octet::new([
///     Trit::True,    // High curiosity
///     Trit::False,   // Low preservation (willing to forget)
///     Trit::Unknown, // Neutral efficiency
///     Trit::Unknown, // Neutral empathy
///     Trit::False,   // Flexible (low rigidity)
///     Trit::True,    // High volatility
///     Trit::True,    // Aggressive cleanup
///     Trit::False,   // Impatient
/// ]);
///
/// let calm_preserving = Octet::new([
///     Trit::False,   // Low curiosity
///     Trit::True,    // High preservation
///     Trit::True,    // Efficient
///     Trit::True,    // Empathetic
///     Trit::True,    // Rigid
///     Trit::False,   // Stable
///     Trit::False,   // Non-aggressive
///     Trit::True,    // Patient
/// ]);
///
/// // These personalities are mostly opposed -> negative resonance
/// let resonance = curious_aggressive.resonance(&calm_preserving);
/// assert!(resonance < 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Octet {
    /// The 8 ternary dimensions
    pub values: [Trit; 8],
}

impl Octet {
    /// Create a new Octet from raw values
    #[inline]
    pub const fn new(values: [Trit; 8]) -> Self {
        Self { values }
    }

    /// Create a neutral "Tabula Rasa" personality (all Unknown)
    #[inline]
    pub const fn neutral() -> Self {
        Self {
            values: [Trit::Unknown; 8],
        }
    }

    /// Calculate resonance (compatibility) between two Octets.
    ///
    /// Returns a value in range `[-1.0, +1.0]`:
    /// - `+1.0`: Perfect alignment (all active dimensions match)
    /// - `0.0`: No correlation (orthogonal or all Unknown)
    /// - `-1.0`: Perfect opposition (all active dimensions conflict)
    ///
    /// Unknown dimensions are excluded from calculation (sparse logic).
    pub fn resonance(&self, other: &Octet) -> f64 {
        let mut dot_product: i32 = 0;
        let mut active_dimensions: i32 = 0;

        for i in 0..8 {
            let a = self.values[i];
            let b = other.values[i];

            // Skip if either party has "no opinion" on this dimension
            if a != Trit::Unknown && b != Trit::Unknown {
                dot_product += (a * b).weight() as i32;
                active_dimensions += 1;
            }
        }

        if active_dimensions == 0 {
            0.0
        } else {
            dot_product as f64 / active_dimensions as f64
        }
    }

    /// Calculate dissonance (conflict magnitude) between two Octets.
    ///
    /// Returns the absolute value of negative resonance, or 0 if resonance is positive.
    /// High dissonance indicates cognitive stress / incompatibility.
    #[inline]
    pub fn dissonance(&self, other: &Octet) -> f64 {
        let res = self.resonance(other);
        if res < 0.0 { res.abs() } else { 0.0 }
    }

    /// Get a specific dimension by index
    #[inline]
    pub const fn get(&self, dim: usize) -> Trit {
        self.values[dim]
    }

    /// Set a specific dimension
    #[inline]
    pub fn set(&mut self, dim: usize, value: Trit) {
        self.values[dim] = value;
    }

    /// Pack the Octet into a single u64 for compact storage.
    ///
    /// Each Trit uses 2 bits: 00=Unknown, 01=True, 11=False
    /// Total: 16 bits used (8 dimensions × 2 bits)
    pub fn pack(&self) -> u16 {
        let mut packed: u16 = 0;
        for (i, trit) in self.values.iter().enumerate() {
            let bits: u16 = match trit {
                Trit::Unknown => 0b00,
                Trit::True => 0b01,
                Trit::False => 0b11,
            };
            packed |= bits << (i * 2);
        }
        packed
    }

    /// Unpack from compact u16 representation
    pub fn unpack(packed: u16) -> Self {
        let mut values = [Trit::Unknown; 8];
        for (i, trit) in values.iter_mut().enumerate() {
            let bits = (packed >> (i * 2)) & 0b11;
            *trit = match bits {
                0b00 => Trit::Unknown,
                0b01 => Trit::True,
                0b11 => Trit::False,
                _ => Trit::Unknown, // 0b10 is invalid, treat as Unknown
            };
        }
        Self { values }
    }
}

impl Default for Octet {
    fn default() -> Self {
        Self::neutral()
    }
}

// ============================================================================
// 3. THE GATEKEEPER: QUANTIZER
// ============================================================================

/// Analog-to-Digital Quantizer with Mood Modulation
///
/// Converts continuous floating-point values to discrete Trit states.
/// The conversion threshold is dynamically adjusted based on a "mood" modifier,
/// modeling how emotional state affects decision thresholds.
///
/// # Examples
///
/// ```
/// use mindfry::setun::{Quantizer, Trit};
///
/// let q = Quantizer::new(0.5);
///
/// // Neutral mood: value must exceed 0.5 for True
/// assert_eq!(q.quantize(0.4, 0.0), Trit::Unknown);
/// assert_eq!(q.quantize(0.6, 0.0), Trit::True);
///
/// // Excited mood (+1.0): threshold drops to 0.4
/// assert_eq!(q.quantize(0.45, 1.0), Trit::True);
///
/// // Stressed mood (-1.0): threshold rises to 0.6
/// assert_eq!(q.quantize(0.55, -1.0), Trit::Unknown);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantizer {
    base_threshold: f64,
}

impl Quantizer {
    /// Create a new Quantizer with the specified base threshold.
    ///
    /// Recommended value: 0.33 (tercile split) or 0.5 (median split)
    #[inline]
    pub const fn new(base_threshold: f64) -> Self {
        Self { base_threshold }
    }

    /// Quantize an analog value to a Trit with mood modulation.
    ///
    /// # Arguments
    /// * `value` - The analog value to quantize (typically 0.0 to 1.0 or -1.0 to 1.0)
    /// * `mood_modifier` - Emotional state modifier (-1.0 to +1.0)
    ///   - Positive: Excited/Optimistic → lower threshold (easier "yes")
    ///   - Negative: Stressed/Pessimistic → higher threshold (harder "yes")
    ///   - Zero: Neutral/Stoic (default)
    pub fn quantize(&self, value: f64, mood_modifier: f64) -> Trit {
        // Mood effect: shift threshold by up to ±10%
        // Clamp to minimum 0.01 to prevent division issues
        let dynamic_threshold = (self.base_threshold - (mood_modifier * 0.1)).max(0.01);

        if value > dynamic_threshold {
            Trit::True
        } else if value < -dynamic_threshold {
            Trit::False
        } else {
            Trit::Unknown
        }
    }

    /// Get the current base threshold
    #[inline]
    pub const fn threshold(&self) -> f64 {
        self.base_threshold
    }
}

impl Default for Quantizer {
    fn default() -> Self {
        Self::new(0.33)
    }
}

// ============================================================================
// 4. THE BRAIN: CORTEX
// ============================================================================

/// The Cortex - MindFry's Decision-Making Brain
///
/// Encapsulates the system's personality, emotional state, and decision logic.
/// Uses balanced ternary (Trit) for nuanced three-state decisions.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │               CORTEX                    │
/// ├─────────────────────────────────────────┤
/// │  personality: Octet  (DNA - immutable)  │
/// │  mood: f64           (State - mutable)  │
/// │  quantizer: Quantizer (Config)          │
/// └─────────────────────────────────────────┘
/// ```
///
/// # Example
///
/// ```
/// use mindfry::setun::{Cortex, Octet, Trit, dimension};
///
/// // Create a curious, preserving personality
/// let mut personality = Octet::neutral();
/// personality.set(dimension::CURIOSITY, Trit::True);
/// personality.set(dimension::PRESERVATION, Trit::True);
///
/// let mut cortex = Cortex::new(personality);
///
/// // Evaluate an event
/// let mut event = Octet::neutral();
/// event.set(dimension::CURIOSITY, Trit::True);
/// let resonance = cortex.evaluate(&event);
/// assert!(resonance > 0.0);  // Positive resonance - compatible
///
/// // Mood affects decisions
/// cortex.shift_mood(-0.5);  // Become stressed
/// assert!(cortex.mood() < 0.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cortex {
    /// The system's fixed personality (DNA)
    personality: Octet,
    /// Current emotional state (-1.0 stressed to +1.0 excited)
    mood: f64,
    /// Threshold modulator for analog-to-trit conversion
    quantizer: Quantizer,
    /// Data retention buffer for safe garbage collection
    retention: RetentionBuffer,
}

impl Cortex {
    /// Create a new Cortex with the given personality.
    ///
    /// Starts with neutral mood (0.0) and default quantizer threshold (0.33).
    pub fn new(personality: Octet) -> Self {
        Self {
            personality,
            mood: 0.0,
            quantizer: Quantizer::default(),
            retention: RetentionBuffer::default(),
        }
    }

    /// Create a new Cortex with custom quantizer threshold.
    pub fn with_threshold(personality: Octet, threshold: f64) -> Self {
        Self {
            personality,
            mood: 0.0,
            quantizer: Quantizer::new(threshold),
            retention: RetentionBuffer::default(),
        }
    }

    /// Get the current mood (-1.0 to +1.0)
    #[inline]
    pub fn mood(&self) -> f64 {
        self.mood
    }

    /// Get a reference to the personality
    #[inline]
    pub fn personality(&self) -> &Octet {
        &self.personality
    }

    /// Shift the mood by a delta, clamped to [-1.0, +1.0].
    ///
    /// Positive delta = more excited/optimistic
    /// Negative delta = more stressed/pessimistic
    #[inline]
    pub fn shift_mood(&mut self, delta: f64) {
        self.mood = (self.mood + delta).clamp(-1.0, 1.0);
    }

    /// Set the mood directly (external override).
    ///
    /// Used for MFBP `MOOD_SET` command from NABU.
    #[inline]
    pub fn set_mood(&mut self, value: f64) {
        self.mood = value.clamp(-1.0, 1.0);
    }

    /// Evaluate an event/entity against the system personality.
    ///
    /// Returns resonance score in [-1.0, +1.0]:
    /// - Positive: Compatible, aligned
    /// - Zero: Neutral, no opinion
    /// - Negative: Conflicting, opposed
    #[inline]
    pub fn evaluate(&self, event: &Octet) -> f64 {
        self.personality.resonance(event)
    }

    /// Make a ternary decision based on an analog value.
    ///
    /// The decision is influenced by the current mood:
    /// - Positive mood → easier to decide "True"
    /// - Negative mood → harder to decide "True"
    ///
    /// # Arguments
    /// * `value` - The analog input to quantize (e.g., energy level)
    #[inline]
    pub fn decide(&self, value: f64) -> Trit {
        self.quantizer.quantize(value, self.mood)
    }

    /// Make a consciousness decision for a lineage.
    ///
    /// Converts the binary is_conscious check into a ternary state:
    /// - True (+1): Lucid - fully conscious (energy significantly above threshold)
    /// - Unknown (0): Dreaming - semi-conscious (energy near threshold)
    /// - False (-1): Dormant - not conscious (energy below threshold)
    ///
    /// **Mood-Coupled Sensitivity:**
    /// - Euphoric mood (+1.0) → higher sensitivity (1.5x) → notices subtle changes
    /// - Neutral mood (0.0) → base sensitivity (1.0x)
    /// - Stressed mood (-1.0) → lower sensitivity (0.5x) → "numb" to stimuli
    ///
    /// # Arguments
    /// * `energy` - Current energy level (0.0 to 1.0)
    /// * `threshold` - The lineage's consciousness threshold
    pub fn consciousness_state(&self, energy: f64, threshold: f64) -> Trit {
        let delta = energy - threshold;

        // Dynamic Gain: Mood affects how sensitive we are to energy changes
        // Base sensitivity of 5.0 gives cleaner mood modulation than 10.0
        const BASE_SENSITIVITY: f64 = 5.0;

        // Mood modifier: +1.0 mood → 1.5x, -1.0 mood → 0.5x
        // Formula: 1.0 + (mood * 0.5) gives range [0.5, 1.5]
        let sensitivity_modifier = 1.0 + (self.mood * 0.5);
        let dynamic_gain = BASE_SENSITIVITY * sensitivity_modifier;

        // Amplify signal with mood-adjusted gain
        let amplified = delta * dynamic_gain;

        // Quantizer also uses mood to shift thresholds (double modulation)
        self.quantizer.quantize(amplified, self.mood)
    }

    /// Get a mutable reference to the retention buffer
    #[inline]
    pub fn retention_mut(&mut self) -> &mut RetentionBuffer {
        &mut self.retention
    }

    /// Get the retention buffer stats
    #[inline]
    pub fn pending_removal_count(&self) -> usize {
        self.retention.pending_count()
    }
}

impl Default for Cortex {
    fn default() -> Self {
        Self::new(Octet::neutral())
    }
}

// ============================================================================
// 5. THE BUFFER: RETENTION BUFFER (Data Lifecycle Management)
// ============================================================================

/// RetentionBuffer - TTL-based data retention before garbage collection.
///
/// When the system decides a lineage should be removed (`Trit::False`) or
/// is uncertain (`Trit::Unknown`), it doesn't delete immediately. Instead,
/// it places the lineage in this buffer with a TTL (Time-To-Live).
///
/// **Lifecycle:**
/// 1. Lineage marked for removal → enters buffer with `default_ttl`
/// 2. Each GC tick → TTL decrements
/// 3. If lineage is stimulated → `restore()` removes it from buffer
/// 4. If TTL reaches 0 → safe to delete
///
/// This prevents "mass extinction" events and provides a debounce mechanism.
///
/// # Example
///
/// ```
/// use mindfry::setun::RetentionBuffer;
///
/// let mut buffer = RetentionBuffer::new(3);  // 3 tick TTL
///
/// // First mark - TTL set to 3
/// assert!(!buffer.mark_or_tick(42));  // Not ready for deletion
///
/// // Tick down - TTL now 2
/// assert!(!buffer.mark_or_tick(42));
///
/// // Restore - lineage was stimulated
/// buffer.restore(42);
/// assert_eq!(buffer.pending_count(), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionBuffer {
    /// Key: Lineage index, Value: Remaining TTL (ticks until deletion)
    pending_removals: std::collections::HashMap<usize, u8>,
    /// Default TTL for new entries
    default_ttl: u8,
}

impl RetentionBuffer {
    /// Create a new retention buffer with specified TTL.
    ///
    /// # Arguments
    /// * `ttl` - Number of GC ticks before deletion is allowed
    pub fn new(ttl: u8) -> Self {
        Self {
            pending_removals: std::collections::HashMap::new(),
            default_ttl: ttl,
        }
    }

    /// Mark a lineage for removal or tick its TTL.
    ///
    /// - If not in buffer: adds with `default_ttl`
    /// - If in buffer: decrements TTL
    /// - Returns `true` if TTL has expired (safe to delete)
    /// - Returns `false` if still in retention period
    pub fn mark_or_tick(&mut self, id: usize) -> bool {
        let ttl = self.pending_removals.entry(id).or_insert(self.default_ttl);

        if *ttl > 0 {
            *ttl -= 1;
            false // Still in buffer, don't delete
        } else {
            self.pending_removals.remove(&id);
            true // TTL expired, safe to delete
        }
    }

    /// Restore a lineage from the buffer (it recovered).
    ///
    /// Called when a lineage is stimulated or otherwise becomes healthy again.
    pub fn restore(&mut self, id: usize) {
        self.pending_removals.remove(&id);
    }

    /// Check if a lineage is pending removal.
    #[inline]
    pub fn is_pending(&self, id: usize) -> bool {
        self.pending_removals.contains_key(&id)
    }

    /// Get the remaining TTL for a pending lineage.
    #[inline]
    pub fn remaining_ttl(&self, id: usize) -> Option<u8> {
        self.pending_removals.get(&id).copied()
    }

    /// Number of lineages pending removal.
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending_removals.len()
    }

    /// Clear all pending removals (reset buffer).
    pub fn clear(&mut self) {
        self.pending_removals.clear();
    }

    /// Get default TTL value.
    #[inline]
    pub fn default_ttl(&self) -> u8 {
        self.default_ttl
    }
}

impl Default for RetentionBuffer {
    fn default() -> Self {
        Self::new(3) // 3 ticks default
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Trit Tests ---

    #[test]
    fn test_trit_consensus() {
        // Same sign strengthens
        assert_eq!(Trit::True * Trit::True, Trit::True);
        assert_eq!(Trit::False * Trit::False, Trit::True);

        // Opposite signs cancel to negative
        assert_eq!(Trit::True * Trit::False, Trit::False);
        assert_eq!(Trit::False * Trit::True, Trit::False);

        // Unknown absorbs
        assert_eq!(Trit::Unknown * Trit::True, Trit::Unknown);
        assert_eq!(Trit::Unknown * Trit::False, Trit::Unknown);
        assert_eq!(Trit::Unknown * Trit::Unknown, Trit::Unknown);
    }

    #[test]
    fn test_trit_inversion() {
        assert_eq!(!Trit::True, Trit::False);
        assert_eq!(!Trit::False, Trit::True);
        assert_eq!(!Trit::Unknown, Trit::Unknown);
    }

    #[test]
    fn test_trit_weight() {
        assert_eq!(Trit::False.weight(), -1);
        assert_eq!(Trit::Unknown.weight(), 0);
        assert_eq!(Trit::True.weight(), 1);
    }

    #[test]
    fn test_trit_from_i8() {
        assert_eq!(Trit::from_i8_saturating(5), Trit::True);
        assert_eq!(Trit::from_i8_saturating(1), Trit::True);
        assert_eq!(Trit::from_i8_saturating(0), Trit::Unknown);
        assert_eq!(Trit::from_i8_saturating(-1), Trit::False);
        assert_eq!(Trit::from_i8_saturating(-100), Trit::False);
    }

    // --- Octet Tests ---

    #[test]
    fn test_octet_perfect_resonance() {
        let p1 = Octet::new([Trit::True; 8]);
        let p2 = Octet::new([Trit::True; 8]);
        assert!((p1.resonance(&p2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_octet_perfect_opposition() {
        let p1 = Octet::new([Trit::True; 8]);
        let p2 = Octet::new([Trit::False; 8]);
        assert!((p1.resonance(&p2) - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_octet_neutral_resonance() {
        let p1 = Octet::neutral();
        let p2 = Octet::new([Trit::True; 8]);
        // All Unknown in p1 → no active dimensions → 0.0
        assert_eq!(p1.resonance(&p2), 0.0);
    }

    #[test]
    fn test_octet_partial_resonance() {
        // Half agree, half disagree → 0.0
        let mut vals1 = [Trit::Unknown; 8];
        vals1[0] = Trit::True;
        vals1[1] = Trit::True;
        vals1[2] = Trit::False;
        vals1[3] = Trit::False;
        let p1 = Octet::new(vals1);

        let mut vals2 = [Trit::Unknown; 8];
        vals2[0] = Trit::True; // Agree
        vals2[1] = Trit::False; // Disagree
        vals2[2] = Trit::True; // Disagree
        vals2[3] = Trit::False; // Agree
        let p2 = Octet::new(vals2);

        // (1 + -1 + -1 + 1) / 4 = 0
        assert_eq!(p1.resonance(&p2), 0.0);
    }

    #[test]
    fn test_octet_dissonance() {
        let p1 = Octet::new([Trit::True; 8]);
        let p2 = Octet::new([Trit::False; 8]);

        // Perfect opposition → dissonance = 1.0
        assert!((p1.dissonance(&p2) - 1.0).abs() < f64::EPSILON);

        // Perfect agreement → dissonance = 0.0
        let p3 = Octet::new([Trit::True; 8]);
        assert_eq!(p1.dissonance(&p3), 0.0);
    }

    #[test]
    fn test_octet_pack_unpack() {
        let original = Octet::new([
            Trit::True,
            Trit::False,
            Trit::Unknown,
            Trit::True,
            Trit::False,
            Trit::Unknown,
            Trit::True,
            Trit::False,
        ]);

        let packed = original.pack();
        let unpacked = Octet::unpack(packed);

        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_octet_dimension_access() {
        let mut o = Octet::neutral();
        o.set(dimension::CURIOSITY, Trit::True);
        o.set(dimension::AGGRESSION, Trit::False);

        assert_eq!(o.get(dimension::CURIOSITY), Trit::True);
        assert_eq!(o.get(dimension::AGGRESSION), Trit::False);
        assert_eq!(o.get(dimension::EMPATHY), Trit::Unknown);
    }

    // --- Quantizer Tests ---

    #[test]
    fn test_quantizer_neutral_mood() {
        let q = Quantizer::new(0.5);

        assert_eq!(q.quantize(0.6, 0.0), Trit::True);
        assert_eq!(q.quantize(0.4, 0.0), Trit::Unknown);
        assert_eq!(q.quantize(-0.6, 0.0), Trit::False);
    }

    #[test]
    fn test_quantizer_positive_mood() {
        let q = Quantizer::new(0.5);

        // Mood +1.0 → threshold drops from 0.5 to 0.4
        // 0.45 > 0.4 → True
        assert_eq!(q.quantize(0.45, 1.0), Trit::True);

        // 0.35 < 0.4 → Unknown
        assert_eq!(q.quantize(0.35, 1.0), Trit::Unknown);
    }

    #[test]
    fn test_quantizer_negative_mood() {
        let q = Quantizer::new(0.5);

        // Mood -1.0 → threshold rises from 0.5 to 0.6
        // 0.55 < 0.6 → Unknown
        assert_eq!(q.quantize(0.55, -1.0), Trit::Unknown);

        // 0.65 > 0.6 → True
        assert_eq!(q.quantize(0.65, -1.0), Trit::True);
    }

    #[test]
    fn test_quantizer_default() {
        let q = Quantizer::default();
        assert!((q.threshold() - 0.33).abs() < f64::EPSILON);
    }

    // --- Cortex Tests ---

    #[test]
    fn test_cortex_creation() {
        let cortex = Cortex::default();
        assert_eq!(cortex.mood(), 0.0);
        assert_eq!(*cortex.personality(), Octet::neutral());
    }

    #[test]
    fn test_cortex_mood_shift() {
        let mut cortex = Cortex::default();

        cortex.shift_mood(0.5);
        assert!((cortex.mood() - 0.5).abs() < f64::EPSILON);

        cortex.shift_mood(0.8); // Would be 1.3, clamped to 1.0
        assert!((cortex.mood() - 1.0).abs() < f64::EPSILON);

        cortex.shift_mood(-2.5); // Would be -1.5, clamped to -1.0
        assert!((cortex.mood() - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cortex_mood_set() {
        let mut cortex = Cortex::default();

        cortex.set_mood(0.75);
        assert!((cortex.mood() - 0.75).abs() < f64::EPSILON);

        // External override with clamping
        cortex.set_mood(2.0);
        assert!((cortex.mood() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cortex_evaluate() {
        let mut personality = Octet::neutral();
        personality.set(dimension::CURIOSITY, Trit::True);
        personality.set(dimension::PRESERVATION, Trit::True);

        let cortex = Cortex::new(personality);

        // Compatible event
        let mut event = Octet::neutral();
        event.set(dimension::CURIOSITY, Trit::True);
        assert!(cortex.evaluate(&event) > 0.0);

        // Conflicting event
        let mut conflict = Octet::neutral();
        conflict.set(dimension::CURIOSITY, Trit::False);
        conflict.set(dimension::PRESERVATION, Trit::False);
        assert!(cortex.evaluate(&conflict) < 0.0);
    }

    #[test]
    fn test_cortex_decide() {
        let cortex = Cortex::default();

        // High value → True
        assert_eq!(cortex.decide(0.8), Trit::True);

        // Low value → Unknown (within threshold)
        assert_eq!(cortex.decide(0.2), Trit::Unknown);

        // Negative value → False
        assert_eq!(cortex.decide(-0.5), Trit::False);
    }

    #[test]
    fn test_cortex_consciousness_state() {
        let cortex = Cortex::default(); // Neutral mood (0.0), gain = 5.0 * 1.0 = 5.0

        // Well above threshold (0.9 - 0.5 = 0.4 * 5 = 2.0) → Lucid
        assert_eq!(cortex.consciousness_state(0.9, 0.5), Trit::True);

        // Well below threshold (0.2 - 0.5 = -0.3 * 5 = -1.5) → Dormant
        assert_eq!(cortex.consciousness_state(0.2, 0.5), Trit::False);

        // Above threshold but closer (0.6 - 0.5 = 0.1 * 5 = 0.5 > 0.33) → Lucid
        assert_eq!(cortex.consciousness_state(0.6, 0.5), Trit::True);

        // Very close to threshold (0.52 - 0.5 = 0.02 * 5 = 0.1 < 0.33) → Dreaming
        assert_eq!(cortex.consciousness_state(0.52, 0.5), Trit::Unknown);

        // Fix for 0.95 bug: (0.95 - 0.8 = 0.15 * 5 = 0.75 > 0.33) → Lucid
        assert_eq!(cortex.consciousness_state(0.95, 0.8), Trit::True);
    }

    #[test]
    fn test_cortex_mood_affects_decision() {
        let mut cortex = Cortex::with_threshold(Octet::neutral(), 0.5);

        // Neutral mood: 0.45 is below 0.5 threshold → Unknown
        assert_eq!(cortex.decide(0.45), Trit::Unknown);

        // Excited mood (+1.0): threshold drops to 0.4, 0.45 > 0.4 → True
        cortex.set_mood(1.0);
        assert_eq!(cortex.decide(0.45), Trit::True);

        // Stressed mood (-1.0): threshold rises to 0.6, 0.55 < 0.6 → Unknown
        cortex.set_mood(-1.0);
        assert_eq!(cortex.decide(0.55), Trit::Unknown);
    }

    // --- RetentionBuffer Tests ---

    #[test]
    fn test_retention_buffer_creation() {
        let buffer = RetentionBuffer::new(5);
        assert_eq!(buffer.default_ttl(), 5);
        assert_eq!(buffer.pending_count(), 0);
    }

    #[test]
    fn test_retention_buffer_mark_and_tick() {
        let mut buffer = RetentionBuffer::new(3);

        // First mark - TTL set to 3, then decremented to 2
        assert!(!buffer.mark_or_tick(42));
        assert!(buffer.is_pending(42));
        assert_eq!(buffer.remaining_ttl(42), Some(2));

        // Second tick - TTL now 1
        assert!(!buffer.mark_or_tick(42));
        assert_eq!(buffer.remaining_ttl(42), Some(1));

        // Third tick - TTL now 0
        assert!(!buffer.mark_or_tick(42));
        assert_eq!(buffer.remaining_ttl(42), Some(0));

        // Fourth tick - TTL expired, safe to delete
        assert!(buffer.mark_or_tick(42));
        assert!(!buffer.is_pending(42));
    }

    #[test]
    fn test_retention_buffer_restore() {
        let mut buffer = RetentionBuffer::new(3);

        // Mark for removal
        buffer.mark_or_tick(100);
        assert!(buffer.is_pending(100));

        // Restore (data recovered)
        buffer.restore(100);
        assert!(!buffer.is_pending(100));
        assert_eq!(buffer.pending_count(), 0);
    }

    #[test]
    fn test_retention_buffer_multiple_items() {
        let mut buffer = RetentionBuffer::new(2);

        buffer.mark_or_tick(1);
        buffer.mark_or_tick(2);
        buffer.mark_or_tick(3);

        assert_eq!(buffer.pending_count(), 3);

        buffer.restore(2);
        assert_eq!(buffer.pending_count(), 2);

        buffer.clear();
        assert_eq!(buffer.pending_count(), 0);
    }

    #[test]
    fn test_cortex_has_retention_buffer() {
        let mut cortex = Cortex::default();

        // Cortex should have a retention buffer
        assert_eq!(cortex.pending_removal_count(), 0);

        // Mark something for removal via cortex
        cortex.retention_mut().mark_or_tick(99);
        assert_eq!(cortex.pending_removal_count(), 1);

        // Restore it
        cortex.retention_mut().restore(99);
        assert_eq!(cortex.pending_removal_count(), 0);
    }
}
