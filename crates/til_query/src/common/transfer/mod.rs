use std::convert::TryFrom;

use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::NonNegative,
};

use crate::common::transfer::utils::bits_from_str;

use self::element::Element;

pub mod element;
pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transfer<
    const ELEMENT_SIZE: usize,
    const USER_SIZE: usize,
    const MAX_DIMENSION: NonNegative,
    const LANES: usize,
> {
    /// The lanes of the physical stream used in this transfer, consisting of
    /// a number of elements.
    ///
    /// The number of lanes must be equal to or less than the number of lanes on
    /// the physical stream.
    lanes: [Element<ELEMENT_SIZE, MAX_DIMENSION>; LANES],
    user: [bool; USER_SIZE],
}

impl<
        const ELEMENT_SIZE: usize,
        const USER_SIZE: usize,
        const MAX_DIMENSION: NonNegative,
        const LANES: usize,
    > Transfer<ELEMENT_SIZE, USER_SIZE, MAX_DIMENSION, LANES>
{
    pub fn new(
        lanes: [Element<ELEMENT_SIZE, MAX_DIMENSION>; LANES],
        user: [bool; USER_SIZE],
    ) -> Self {
        Self { lanes, user }
    }

    pub fn lanes(&self) -> &[Element<ELEMENT_SIZE, MAX_DIMENSION>; LANES] {
        &self.lanes
    }
}

fn try_elements_from_array<
    const ELEMENT_SIZE: usize,
    const MAX_DIMENSION: NonNegative,
    const LANES: usize,
    E: TryResult<Element<ELEMENT_SIZE, MAX_DIMENSION>>,
>(
    value: [E; LANES],
) -> Result<[Element<ELEMENT_SIZE, MAX_DIMENSION>; LANES]> {
    let lanes = value.map(|x| x.try_result());
    // array_try_map is currently unstable, so this'll have to do for now.
    if lanes.iter().any(|x| x.is_err()) {
        lanes
            .into_iter()
            .find_map(|x| match x {
                Err(err) => Some(Err(err)),
                _ => unreachable!(),
            })
            .unwrap()
    } else {
        Ok(lanes.map(|x| x.unwrap()))
    }
}

impl<const ELEMENT_SIZE: usize, const MAX_DIMENSION: NonNegative, const LANES: usize, E>
    TryFrom<[E; LANES]> for Transfer<ELEMENT_SIZE, 0, MAX_DIMENSION, LANES>
where
    E: TryResult<Element<ELEMENT_SIZE, MAX_DIMENSION>>,
{
    type Error = Error;

    fn try_from(value: [E; LANES]) -> Result<Self> {
        Ok(Self::new(try_elements_from_array(value)?, []))
    }
}

impl<
        'a,
        const ELEMENT_SIZE: usize,
        const USER_SIZE: usize,
        const MAX_DIMENSION: NonNegative,
        const LANES: usize,
        E,
    > TryFrom<([E; LANES], &'a str)> for Transfer<ELEMENT_SIZE, USER_SIZE, MAX_DIMENSION, LANES>
where
    E: TryResult<Element<ELEMENT_SIZE, MAX_DIMENSION>>,
{
    type Error = Error;

    fn try_from(value: ([E; LANES], &'a str)) -> Result<Self> {
        Ok(Self::new(
            try_elements_from_array(value.0)?,
            bits_from_str(value.1)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tydi_common::numbers::Positive;

    use super::*;

    #[test]
    fn try_result_compare() -> Result<()> {
        let transfer_1: Transfer<4, 0, 0, 2> = Transfer::new(
            [
                Element::new_data([true, false, true, true]),
                Element::new_data([true, false, false, false]),
            ],
            [],
        );
        let transfer_2 = ["1101", "0001"].try_result()?;

        assert_eq!(transfer_1, transfer_2);

        let transfer_3: Transfer<4, 2, 0, 2> = Transfer::new(
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

    struct SomeContainer {
        el_size: NonNegative,
        user_size: NonNegative,
        max_dim: NonNegative,
        lanes: Positive,
    }

    impl SomeContainer {
        fn new(
            el_size: NonNegative,
            user_size: NonNegative,
            max_dim: NonNegative,
            lanes: Positive,
        ) -> Self {
            Self {
                el_size,

                user_size,
                max_dim,
                lanes,
            }
        }

        fn lanes(&self) -> &Positive {
            &self.lanes
        }

        fn el_size(&self) -> NonNegative {
            self.el_size
        }

        fn max_dim(&self) -> NonNegative {
            self.max_dim
        }

        fn user_size(&self) -> NonNegative {
            self.user_size
        }
    }

    #[test]
    fn validate_const_behaviour() -> Result<()> {
        let container = SomeContainer::new(2, 2, 2, 2);

        let el_size = container.el_size();
        let user_size = container.user_size();
        let max_dim = container.max_dim();
        let lanes = container.lanes().get();

        let transfer: Transfer<el_size, user_size, max_dim, lanes> =
            (["11", "01"], "10").try_result()?;

        Ok(())
    }
}
