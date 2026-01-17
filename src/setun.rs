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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone)]
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
}
