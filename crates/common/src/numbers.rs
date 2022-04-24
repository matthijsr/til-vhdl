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

pub fn u32_to_i32(u: u32) -> Result<i32> {
    i32::try_from(u).map_err(|err| Error::InvalidArgument(err.to_string()))
}

pub fn i32_to_u32(i: i32) -> Result<u32> {
    u32::try_from(i).map_err(|err| Error::InvalidArgument(err.to_string()))
}

pub fn usize_to_u32(u: usize) -> Result<u32> {
    u32::try_from(u).map_err(|err| Error::InvalidArgument(err.to_string()))
}

pub fn u32_to_usize(u: u32) -> Result<usize> {
    usize::try_from(u).map_err(|err| Error::InvalidArgument(err.to_string()))
}
