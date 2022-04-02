use std::{convert::TryFrom, ops::Range, str::FromStr};

use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::NonNegative,
};

use super::{element::Element, element_type::ElementType};

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
    ///
    /// If None, this Transfer does not address the user signal.
    user: Option<ElementType>,
}

impl LogicalTransfer {
    pub fn new(
        lanes: impl IntoIterator<Item = Element>,
        user: Option<impl Into<ElementType>>,
    ) -> Self {
        Self {
            data: LogicalData::Lanes(lanes.into_iter().collect()),
            user: user.map(|x| x.into()),
        }
    }

    /// Create a new empty sequence without a user transfer.
    pub fn new_empty(last: Range<NonNegative>) -> Self {
        Self {
            data: LogicalData::EmptySequence(last),
            user: None,
        }
    }

    /// Create a new empty sequence with a user transfer
    pub fn new_empty_user(last: Range<NonNegative>, user: impl Into<ElementType>) -> Self {
        Self {
            data: LogicalData::EmptySequence(last),
            user: Some(user.into()),
        }
    }

    pub fn try_new(
        lanes: impl IntoIterator<Item = impl TryResult<Element>>,
        user: Option<impl TryResult<ElementType>>,
    ) -> Result<Self> {
        let user = if let Some(user) = user {
            Some(user.try_result()?)
        } else {
            None
        };
        Ok(Self {
            data: LogicalData::Lanes(
                lanes
                    .into_iter()
                    .map(|x| x.try_result())
                    .collect::<Result<Vec<Element>>>()?,
            ),
            user: user,
        })
    }

    pub fn new_lanes(lanes: impl IntoIterator<Item = Element>) -> Self {
        Self {
            data: LogicalData::Lanes(lanes.into_iter().collect()),
            user: None,
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

    pub fn set_user(&mut self, user: impl Into<ElementType>) {
        self.user = Some(user.into());
    }

    pub fn with_user(mut self, user: impl Into<ElementType>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn data(&self) -> &LogicalData {
        &self.data
    }

    pub fn user(&self) -> &Option<ElementType> {
        &self.user
    }
}

impl From<LogicalData> for LogicalTransfer {
    fn from(value: LogicalData) -> Self {
        Self {
            data: value,
            user: None,
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
        let user = if value.1 == "-" || value.1.to_lowercase() == "inactive" {
            None
        } else {
            Some(ElementType::from_str(value.1)?)
        };

        Self::try_new(value.0, user)
    }
}

impl<'a> TryFrom<(LogicalData, &'a str)> for LogicalTransfer {
    type Error = Error;

    fn try_from(value: (LogicalData, &'a str)) -> Result<Self> {
        let user = if value.1 == "-" || value.1.to_lowercase() == "inactive" {
            None
        } else {
            Some(ElementType::from_str(value.1)?)
        };

        Ok(Self {
            data: value.0,
            user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bitvec::prelude::*;

    #[test]
    fn try_result_compare() -> Result<()> {
        let transfer_1: LogicalTransfer = LogicalTransfer::new(
            [
                Element::new_data(bitvec![1, 0, 1, 1]),
                Element::new_data(bitvec![1, 0, 0, 0]),
            ],
            None::<ElementType>,
        );
        let transfer_2 = ["1011", "1000"].try_result()?;

        assert_eq!(transfer_1, transfer_2);

        let transfer_3: LogicalTransfer = LogicalTransfer::new(
            [
                Element::new_data(bitvec![1, 0, 1, 1]),
                Element::new_data(bitvec![1, 0, 0, 0]),
            ],
            Some(bitvec![0, 1]),
        );
        let transfer_4 = (["1011", "1000"], "01").try_result()?;

        assert_eq!(transfer_3, transfer_4);

        let empty_transfer_1 = LogicalTransfer::new_empty(0..2);
        let empty_transfer_2 = (LogicalData::EmptySequence(0..2)).into();
        assert_eq!(empty_transfer_1, empty_transfer_2);

        let empty_transfer_3 = LogicalTransfer::new_empty_user(2..2, bitvec![0, 1, 0]);
        let empty_transfer_4 = (LogicalData::EmptySequence(2..2), "010").try_result()?;
        assert_eq!(empty_transfer_3, empty_transfer_4);

        Ok(())
    }
}
