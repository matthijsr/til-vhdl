use tydi_common::numbers::NonNegative;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Width {
    /// Non-vectorized single bit.
    Scalar,
    /// Vectorized multiple bits.
    Vector(NonNegative),
}
