use super::numbers::{NonNegative, Positive};

/// Returns ⌈log2(x)⌉.
pub const fn log2_ceil(x: Positive) -> NonNegative {
    8 * std::mem::size_of::<NonNegative>() as NonNegative
        - (x.get() - 1).leading_zeros() as NonNegative
}

/// Concatenate stuff using format with an underscore in between.
/// Useful if the separator ever changes.
#[macro_export]
macro_rules! cat {
    ($a:expr) => {{
        format!("{}", $a)
    }};

    ($a:expr, $($b:expr),+) => {{
        let left : String = format!("{}", $a);
        let right : String = format!("{}", cat!($($b),+));
        if left == "" {
            right
        } else if right == "" {
            left
        } else {
            format!("{}_{}", left, right)
        }
    }};
}
