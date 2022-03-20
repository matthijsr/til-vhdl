use std::{convert::TryFrom, ops::Range};

use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::NonNegative,
};

use crate::common::transfer::utils::bits_from_str;

use super::element::Element;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalData {
    /// Indicates an empty sequence. Therefor, it _must_ contain dimension
    /// information.
    EmptySequence(Range<NonNegative>),
    /// The lanes of the physical stream used in this transfer, consisting of
    /// a number of elements.
    ///
    /// The number of lanes must be equal to or less than the number of lanes on
    /// the physical stream.
    Lanes(Vec<Element>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// This is a Logical Transfer, representing the elements and user signal
/// to be transferred over a physical stream in a single clock cycle.
pub struct LogicalTransfer {
    /// The data carried by this Logical Transfer
    data: LogicalData,
    /// The user signal.
    user: Vec<bool>,
}

impl LogicalTransfer {
    pub fn new(
        lanes: impl IntoIterator<Item = Element>,
        user: impl IntoIterator<Item = bool>,
    ) -> Self {
        Self {
            data: LogicalData::Lanes(lanes.into_iter().collect()),
            user: user.into_iter().collect(),
        }
    }

    /// Create a new empty sequence without a user transfer.
    pub fn new_empty(last: Range<NonNegative>) -> Self {
        Self {
            data: LogicalData::EmptySequence(last),
            user: vec![],
        }
    }

    /// Create a new empty sequence with a user transfer
    pub fn new_empty_user(last: Range<NonNegative>, user: impl IntoIterator<Item = bool>) -> Self {
        Self {
            data: LogicalData::EmptySequence(last),
            user: user.into_iter().collect(),
        }
    }

    pub fn try_new(
        lanes: impl IntoIterator<Item = impl TryResult<Element>>,
        user: impl IntoIterator<Item = bool>,
    ) -> Result<Self> {
        Ok(Self {
            data: LogicalData::Lanes(
                lanes
                    .into_iter()
                    .map(|x| x.try_result())
                    .collect::<Result<Vec<Element>>>()?,
            ),
            user: user.into_iter().collect(),
        })
    }

    pub fn new_lanes(lanes: impl IntoIterator<Item = Element>) -> Self {
        Self {
            data: LogicalData::Lanes(lanes.into_iter().collect()),
            user: vec![],
        }
    }

    pub fn try_new_lanes(lanes: impl IntoIterator<Item = impl TryResult<Element>>) -> Result<Self> {
        Ok(Self::new_lanes(
            lanes
                .into_iter()
                .map(|x| x.try_result())
                .collect::<Result<Vec<Element>>>()?,
        ))
    }

    pub fn set_user(&mut self, user: impl IntoIterator<Item = bool>) {
        self.user = user.into_iter().collect();
    }

    pub fn with_user(mut self, user: impl IntoIterator<Item = bool>) -> Self {
        self.user = user.into_iter().collect();
        self
    }

    pub fn data(&self) -> &LogicalData {
        &self.data
    }

    pub fn user(&self) -> &Vec<bool> {
        &self.user
    }
}

impl From<LogicalData> for LogicalTransfer {
    fn from(value: LogicalData) -> Self {
        Self {
            data: value,
            user: vec![],
        }
    }
}

impl<E: TryResult<Element>> TryFrom<Vec<E>> for LogicalTransfer {
    type Error = Error;

    fn try_from(value: Vec<E>) -> Result<Self> {
        Self::try_new_lanes(value)
    }
}

impl<const SIZE: usize, E: TryResult<Element>> TryFrom<[E; SIZE]> for LogicalTransfer {
    type Error = Error;

    fn try_from(value: [E; SIZE]) -> Result<Self> {
        Self::try_new_lanes(value)
    }
}

impl<'a, I, E> TryFrom<(I, &'a str)> for LogicalTransfer
where
    I: IntoIterator<Item = E>,
    E: TryResult<Element>,
{
    type Error = Error;

    fn try_from(value: (I, &'a str)) -> Result<Self> {
        Self::try_new(value.0, bits_from_str(value.1)?)
    }
}

impl<'a> TryFrom<(LogicalData, &'a str)> for LogicalTransfer {
    type Error = Error;

    fn try_from(value: (LogicalData, &'a str)) -> Result<Self> {
        Ok(Self {
            data: value.0,
            user: bits_from_str(value.1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_result_compare() -> Result<()> {
        let transfer_1: LogicalTransfer = LogicalTransfer::new(
            [
                Element::new_data([true, false, true, true]),
                Element::new_data([true, false, false, false]),
            ],
            [],
        );
        let transfer_2 = ["1011", "1000"].try_result()?;

        assert_eq!(transfer_1, transfer_2);

        let transfer_3: LogicalTransfer = LogicalTransfer::new(
            [
                Element::new_data([true, false, true, true]),
                Element::new_data([true, false, false, false]),
            ],
            [false, true],
        );
        let transfer_4 = (["1011", "1000"], "01").try_result()?;

        assert_eq!(transfer_3, transfer_4);

        let empty_transfer_1 = LogicalTransfer::new_empty(0..2);
        let empty_transfer_2 = (LogicalData::EmptySequence(0..2)).into();
        assert_eq!(empty_transfer_1, empty_transfer_2);

        let empty_transfer_3 = LogicalTransfer::new_empty_user(2..2, [false, true, false]);
        let empty_transfer_4 = (LogicalData::EmptySequence(2..2), "010").try_result()?;
        assert_eq!(empty_transfer_3, empty_transfer_4);

        Ok(())
    }
}
