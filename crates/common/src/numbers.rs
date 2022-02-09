use std::{convert::TryFrom, ops::Mul};

use super::error::{Error, Result};

// Types for positive and non-negative integers.

/// Positive integer.
pub type Positive = std::num::NonZeroU32;
/// Non-negative integer.
pub type NonNegative = u32;
/// Positive real.
pub type PositiveReal = NonZeroReal<f64>;
/// Positive number of bits.
pub type BitCount = Positive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZeroReal<T>(T);

impl<T> NonZeroReal<T>
where
    T: Copy + Into<f64>,
{
    pub fn new(real: T) -> Result<Self> {
        if real.into() > 0. {
            Ok(NonZeroReal(real))
        } else {
            Err(Error::InvalidArgument("real must be positive".to_string()))
        }
    }
}

impl<T> Mul for NonZeroReal<T>
where
    T: Copy + Mul<Output = T> + Into<f64>,
{
    type Output = NonZeroReal<T>;

    fn mul(self, other: NonZeroReal<T>) -> Self::Output {
        NonZeroReal::new(self.0 * other.0).unwrap()
    }
}

impl<T> NonZeroReal<T>
where
    T: Copy,
{
    pub fn get(&self) -> T {
        self.0
    }
}

impl TryFrom<f64> for PositiveReal {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self> {
        PositiveReal::new(value)
    }
}

impl TryFrom<u32> for PositiveReal {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        PositiveReal::new(value as f64)
    }
}

impl From<Positive> for PositiveReal {
    fn from(val: Positive) -> Self {
        NonZeroReal(val.get() as f64)
    }
}
