use std::convert::TryFrom;

use tydi_common::error::{Error, Result, TryResult};

use crate::common::transfer::utils::bits_from_str;

use self::element::Element;

pub mod element;
pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transfer {
    /// The lanes of the physical stream used in this transfer, consisting of
    /// a number of elements.
    ///
    /// The number of lanes must be equal to or less than the number of lanes on
    /// the physical stream.
    lanes: Vec<Element>,
    user: Vec<bool>,
}

impl Transfer {
    pub fn new(
        lanes: impl IntoIterator<Item = Element>,
        user: impl IntoIterator<Item = bool>,
    ) -> Self {
        Self {
            lanes: lanes.into_iter().collect(),
            user: user.into_iter().collect(),
        }
    }

    pub fn try_new(
        lanes: impl IntoIterator<Item = impl TryResult<Element>>,
        user: impl IntoIterator<Item = bool>,
    ) -> Result<Self> {
        Ok(Self {
            lanes: lanes
                .into_iter()
                .map(|x| x.try_result())
                .collect::<Result<Vec<Element>>>()?,
            user: user.into_iter().collect(),
        })
    }

    pub fn new_lanes(lanes: impl IntoIterator<Item = Element>) -> Self {
        Self {
            lanes: lanes.into_iter().collect(),
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

    pub fn lanes(&self) -> &Vec<Element> {
        &self.lanes
    }

    pub fn user(&self) -> &Vec<bool> {
        &self.user
    }
}

impl<const SIZE: usize, E: TryResult<Element>> TryFrom<[E; SIZE]> for Transfer {
    type Error = Error;

    fn try_from(value: [E; SIZE]) -> Result<Self> {
        Self::try_new_lanes(value)
    }
}

impl<'a, I, E> TryFrom<(I, &'a str)> for Transfer
where
    I: IntoIterator<Item = E>,
    E: TryResult<Element>,
{
    type Error = Error;

    fn try_from(value: (I, &'a str)) -> Result<Self> {
        Self::try_new(value.0, bits_from_str(value.1)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_result_compare() -> Result<()> {
        let transfer_1: Transfer = Transfer::new(
            [
                Element::new_data([true, false, true, true]),
                Element::new_data([true, false, false, false]),
            ],
            [],
        );
        let transfer_2 = ["1101", "0001"].try_result()?;

        assert_eq!(transfer_1, transfer_2);

        let transfer_3: Transfer = Transfer::new(
            [
                Element::new_data([true, false, true, true]),
                Element::new_data([true, false, false, false]),
            ],
            [false, true],
        );
        let transfer_4 = (["1101", "0001"], "10").try_result()?;

        assert_eq!(transfer_3, transfer_4);

        Ok(())
    }
}
