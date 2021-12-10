use super::integers::{NonNegative, Positive};

/// Returns ⌈log2(x)⌉.
pub(crate) const fn log2_ceil(x: Positive) -> NonNegative {
    8 * std::mem::size_of::<NonNegative>() as NonNegative
        - (x.get() - 1).leading_zeros() as NonNegative
}
