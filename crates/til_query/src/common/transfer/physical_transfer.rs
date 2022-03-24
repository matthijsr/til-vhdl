use std::ops::Range;

use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::{NonNegative, Positive},
};

use crate::common::physical::complexity::Complexity;

use super::logical_transfer::{LogicalData, LogicalTransfer};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// At some complexities, the `valid` signal must be held '1' until parts or
/// all of the sequence have been transferred. This effectively means that
/// some transfers must occur over consecutive clock cycles.
pub enum HoldValidRule {
    /// Valid may only be set '0' after the entire sequence has been
    /// transferred, ending in a `last` which is all '1's.
    WholeSequence(bool),
    /// Valid may only be set '0' after an innermost sequence has been
    /// been transferred, requiring a `last` for dimension 0.
    InnerSequence(bool),
    /// Valid may be set '0' after every transfer.
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The method by which `last` is transferred.
pub enum LastMode {
    /// This stream has no Dimensionality, so does not assert `last`.
    None,
    /// This stream has Complexity < 8, so asserts `last` per transfer.
    Transfer(Option<Range<NonNegative>>),
    /// This stream has Complexity >= 8, so can assert `last` per element lane.
    Lane(Vec<Option<Range<NonNegative>>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The method by which `last` is transferred.
pub enum StrobeMode {
    /// This stream has no Dimensionality or Complexity >= 8,
    /// so does not require a strobe signal.
    None,
    /// This stream has Complexity < 7, so asserts `strobe` per transfer.
    Transfer(bool),
    /// This stream has Complexity >= 7, so can assert `strobe` per element lane.
    Lane(Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A physical transfer for a given physical stream. Validates and organizes
/// logical transfers based on the physical stream's properties.
pub struct PhysicalTransfer {
    /// The Complexity this transfer adheres to. This value informs whether the
    /// logical transfer is valid.
    complexity: Complexity,
    /// Indicates whether this transfer allows for `valid` to be released.
    /// This depends on the Complexity of the stream.
    ///
    /// * C < 3: `valid` can only be released when lane N-1 has a non-zero `last`.
    /// * C < 2: `valid` can only be released when lane N-1 has a `last` of all 1s.
    holds_valid: HoldValidRule,
    /// The number of element lanes the physical stream has. Also referred to as
    /// N.
    element_lanes: Positive,
    /// The maximum width of each element being transferred.
    max_element_size: NonNegative,
    /// The data being transfered, organized by lane.
    ///
    /// When None, treat all lanes as active, but drive all '0's.
    ///
    /// When Some, but empty or short of the number of element lanes, do not
    /// drive the unaddressed lanes.
    ///
    /// When Some, and certain elements are shorter than the maximum element
    /// size, shift to align to the LSB and do not drive the unaddressed bits.
    data: Option<Vec<Vec<bool>>>,
    /// The dimensionality supported by the physical stream.
    dimensionality: NonNegative,
    /// The `last` signalling for the transfer.
    last: LastMode,
    /// The index of the first active lane.
    ///
    /// Requires: C≥6∧N>1
    ///
    /// May not be N or greater.
    ///
    /// When C < 6, must always be 0.
    start_index: Option<NonNegative>,
    /// The index of the last active lane.
    ///
    /// Requires (C≥5∨D≥1)∧N>1
    ///
    /// * May not be N or greater.
    /// * May not be less than `start_index`.
    ///
    /// When C < 5, and `last` is zero, end index must be N-1.
    end_index: Option<NonNegative>,
    /// The `strb` signal.
    ///
    /// Requires: C≥7∨D≥1
    ///
    /// At C < 7, this is used to indicate whether the transfer is empty.
    /// At C >= 7, this indicates the activity of individual element lanes.
    ///
    /// NOTE: This is an assumption, the original spec claims the cut-off is
    /// C >= 8. But this makes C=7 effectively useless.
    strobe: StrobeMode,
    /// The maximum width of the `user` signal being transferred.
    max_user_size: NonNegative,
    /// The `user` signal
    ///
    /// When None, drive all '0's.
    user: Option<Vec<bool>>,
}

impl PhysicalTransfer {
    /// Creates a default, base transfer based on the given properties.
    ///
    /// This transfer will always be valid for the given properties (barring
    /// `user`-based constraints).
    pub fn new(
        complexity: Complexity,
        element_lanes: Positive,
        max_element_size: NonNegative,
        dimensionality: NonNegative,
        max_user_size: NonNegative,
    ) -> Self {
        let last_mode = if dimensionality >= 1 {
            if complexity >= Complexity::new_major(8) {
                LastMode::Lane(vec![
                    Some(0..(dimensionality - 1));
                    element_lanes.get().try_into().unwrap()
                ])
            } else {
                LastMode::Transfer(Some(0..(dimensionality - 1)))
            }
        } else {
            LastMode::None
        };

        let element_lanes_gt_1 = element_lanes > Positive::new(1).unwrap();

        let start_index = if element_lanes_gt_1 && complexity >= Complexity::new_major(6) {
            Some(0)
        } else {
            None
        };

        let end_index = if element_lanes_gt_1
            && (dimensionality >= 1 || complexity >= Complexity::new_major(5))
        {
            Some(element_lanes.get() - 1)
        } else {
            None
        };

        let strobe = if complexity >= Complexity::new_major(7) {
            StrobeMode::Lane(vec![true; element_lanes.get().try_into().unwrap()])
        } else if dimensionality >= 1 {
            StrobeMode::Transfer(true)
        } else {
            StrobeMode::None
        };

        let holds_valid = if complexity < Complexity::new_major(3) {
            if complexity < Complexity::new_major(2) {
                HoldValidRule::WholeSequence(false)
            } else {
                HoldValidRule::InnerSequence(false)
            }
        } else {
            HoldValidRule::None
        };

        Self {
            complexity,
            holds_valid,
            element_lanes,
            max_element_size,
            data: None,
            dimensionality,
            last: last_mode,
            start_index,
            end_index,
            strobe,
            max_user_size,
            user: None,
        }
    }

    pub fn with_logical_transfer(
        mut self,
        transfer: impl TryResult<LogicalTransfer>,
    ) -> Result<Self> {
        let transfer: LogicalTransfer = transfer.try_result()?;

        match transfer.data() {
            LogicalData::EmptySequence(last) => {
                if last.end >= self.dimensionality() {
                    return Err(Error::InvalidArgument(format!("Cannot assert empty sequence as last in dimension {}, as this physical stream has a dimensionality of {}.", last.end, self.dimensionality())));
                }

                match &mut self.holds_valid {
                    HoldValidRule::WholeSequence(holds_valid) => {
                        if last.end < self.dimensionality - 1 {
                            *holds_valid = true;
                        } else {
                            *holds_valid = false;
                        }
                    }
                    HoldValidRule::InnerSequence(holds_valid) => {
                        *holds_valid = false;
                    }
                    HoldValidRule::None => (),
                }

                self.data = Some(vec![]);

                match &mut self.last {
                    LastMode::None => return Err(Error::InvalidArgument("Attempted to transfer an empty sequence, but physical stream has no dimensionality.".to_string())),
                    LastMode::Transfer(transfer_last) => {
                        *transfer_last = Some(last.clone());
                    },
                    LastMode::Lane(lanes_last) => {
                        for lane_last in &mut lanes_last[1..] {
                            *lane_last = None;
                        }
                        lanes_last[0] = Some(last.clone());
                    },
                }

                match &mut self.strobe {
                    StrobeMode::None => unreachable!(), // Already caught by `last` check.
                    StrobeMode::Transfer(strb) => {
                        *strb = false;
                    }
                    StrobeMode::Lane(strb) => {
                        for lane_strb in strb {
                            *lane_strb = false;
                        }
                    }
                }

                /// NOTE TO SELF: Stai and Endi are probably irrelevant for empty sequences?
                if let Some(stai) = &mut self.start_index {
                    *stai = 0;
                }

                if let Some(endi) = &mut self.end_index {
                    *endi = 0;
                }
            }
            LogicalData::Lanes(elements) => {
                if elements.len() > self.element_lanes().get().try_into().unwrap() {
                    return Err(Error::InvalidArgument(format!(
                        "Cannot transfer {} elements. Physical stream has {} lanes.",
                        elements.len(),
                        self.element_lanes()
                    )));
                }

                let (data_vec, last_vec): (Vec<_>, Vec<_>) = elements
                    .iter()
                    .map(|element| (element.data().clone(), element.last().clone()))
                    .unzip();

                match &mut self.last {
                    LastMode::None => {
                        if last_vec.iter().any(|x| x.is_some()) {
                            return Err(Error::InvalidArgument("Attempted to assert last in a dimension, but physical stream has no dimensionality.".to_string()));
                        }
                    }
                    LastMode::Transfer(transfer_last) => {
                        let mut result = None;

                        for last in last_vec {
                            match last {
                                Some(el_last) => {
                                    if let Some(_) = &mut result {
                                        return Err(Error::InvalidArgument(format!("Cannot assert dimensionality on more than one element lane. Physical stream has complexity {} (< 8).", self.complexity())));
                                    } else {
                                        result = Some(el_last);
                                    }
                                }
                                None => (),
                            }
                        }

                        if let Some(result_last) = &result {
                            if result_last.end >= self.dimensionality {
                                return Err(Error::InvalidArgument(
                                    format!("Cannot assert an element or transfer as last in dimension {}, physical stream has dimensionality {}.", result_last.end, self.dimensionality),
                                ));
                            }
                        }

                        match &mut self.holds_valid {
                            HoldValidRule::WholeSequence(holds_valid) => {
                                if let Some(result_last) = &result {
                                    *holds_valid = result_last.end == self.dimensionality - 1;
                                } else {
                                    *holds_valid = true;
                                }
                            }
                            HoldValidRule::InnerSequence(holds_valid) => {
                                *holds_valid = result.is_none();
                            }
                            HoldValidRule::None => (),
                        }

                        *transfer_last = result;
                    }
                    LastMode::Lane(_) => todo!(),
                }
            }
        }

        self.user = Some(transfer.user().clone());

        Ok(self)
    }

    /// The Complexity this transfer adheres to. This value informs whether the
    /// logical transfer is valid.
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }

    /// Indicates whether this transfer allows for `valid` to be released.
    /// This depends on the Complexity of the stream.
    ///
    /// * C < 3: `valid` can only be released when lane N-1 has a non-zero `last`.
    /// * C < 2: `valid` can only be released when lane N-1 has a `last` of all 1s.
    pub fn holds_valid(&self) -> bool {
        match &self.holds_valid {
            HoldValidRule::None => false,
            HoldValidRule::WholeSequence(val) | HoldValidRule::InnerSequence(val) => *val,
        }
    }

    /// The number of element lanes the physical stream has. Also referred to as
    /// N.
    pub fn element_lanes(&self) -> Positive {
        self.element_lanes
    }

    /// The maximum width of each element being transferred.
    pub fn max_element_size(&self) -> NonNegative {
        self.max_element_size
    }

    /// The data being transfered, organized by lane.
    ///
    /// When None, treat all lanes as active, but drive all '0's.
    ///
    /// When Some, but empty or short of the number of element lanes, do not
    /// drive the unaddressed lanes.
    ///
    /// When Some, and certain elements are shorter than the maximum element
    /// size, shift to align to the LSB and do not drive the unaddressed bits.
    pub fn data(&self) -> &Option<Vec<Vec<bool>>> {
        &self.data
    }

    /// The dimensionality supported by the physical stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// The `last` signalling for the transfer.
    pub fn last(&self) -> &LastMode {
        &self.last
    }

    /// The index of the first active lane.
    ///
    /// Requires: C≥6∧N>1
    ///
    /// May not be N or greater.
    ///
    /// When C < 6, must always be 0.
    pub fn start_index(&self) -> &Option<NonNegative> {
        &self.start_index
    }

    /// The index of the last active lane.
    ///
    /// Requires (C≥5∨D≥1)∧N>1
    ///
    /// * May not be N or greater.
    /// * May not be less than `start_index`.
    ///
    /// When C < 5, and `last` is zero, end index must be N-1.
    pub fn end_index(&self) -> &Option<NonNegative> {
        &self.end_index
    }

    /// The `strb` signal.
    ///
    /// Requires: C≥7∨D≥1
    ///
    /// At C < 7, this is used to indicate whether the transfer is empty.
    /// At C >= 7, this indicates the activity of individual element lanes.
    ///
    /// NOTE: This is an assumption, the original spec claims the cut-off is
    /// C >= 8. But this makes C=7 effectively useless.
    pub fn strobe(&self) -> &StrobeMode {
        &self.strobe
    }

    /// The maximum width of the `user` signal being transferred.
    pub fn max_user_size(&self) -> NonNegative {
        self.max_user_size
    }

    /// The `user` signal
    ///
    /// When None, drive all '0's.
    pub fn user(&self) -> &Option<Vec<bool>> {
        &self.user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_transfer() -> Result<()> {
        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(1),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        );

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &None);
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..2)));
        assert_eq!(physical_transfer.start_index(), &None);
        assert_eq!(physical_transfer.end_index(), &Some(2));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(true));
        assert_eq!(physical_transfer.user(), &None);

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(5),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        );

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &None);
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..2)));
        assert_eq!(physical_transfer.start_index(), &None);
        assert_eq!(physical_transfer.end_index(), &Some(2));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(true));
        assert_eq!(physical_transfer.user(), &None);

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(6),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        );

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &None);
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..2)));
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(2));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(true));
        assert_eq!(physical_transfer.user(), &None);

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(7),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        );

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &None);
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..2)));
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(2));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Lane(vec![true; 3]));
        assert_eq!(physical_transfer.user(), &None);

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(8),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        );

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &None);
        assert_eq!(
            physical_transfer.last(),
            &LastMode::Lane(vec![Some(0..2), Some(0..2), Some(0..2)])
        );
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(2));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Lane(vec![true; 3]));
        assert_eq!(physical_transfer.user(), &None);

        Ok(())
    }

    #[test]
    fn test_empty_sequence() -> Result<()> {
        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(1),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        )
        .with_logical_transfer((LogicalData::EmptySequence(0..1), "101"))?;

        assert_eq!(physical_transfer.holds_valid(), true);
        assert_eq!(physical_transfer.data(), &Some(vec![]));
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..1)));
        assert_eq!(physical_transfer.start_index(), &None);
        assert_eq!(physical_transfer.end_index(), &Some(0));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(physical_transfer.user(), &Some(vec![true, false, true]));

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(2),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        )
        .with_logical_transfer((LogicalData::EmptySequence(0..1), "101"))?;

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &Some(vec![]));
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..1)));
        assert_eq!(physical_transfer.start_index(), &None);
        assert_eq!(physical_transfer.end_index(), &Some(0));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(physical_transfer.user(), &Some(vec![true, false, true]));

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(6),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        )
        .with_logical_transfer((LogicalData::EmptySequence(0..1), "101"))?;

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &Some(vec![]));
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..1)));
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(0));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(physical_transfer.user(), &Some(vec![true, false, true]));

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(7),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        )
        .with_logical_transfer((LogicalData::EmptySequence(0..1), "101"))?;

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &Some(vec![]));
        assert_eq!(physical_transfer.last(), &LastMode::Transfer(Some(0..1)));
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(0));
        assert_eq!(
            physical_transfer.strobe(),
            &StrobeMode::Lane(vec![false; 3])
        );
        assert_eq!(physical_transfer.user(), &Some(vec![true, false, true]));

        let physical_transfer = PhysicalTransfer::new(
            Complexity::new_major(8),
            Positive::new(3).unwrap(),
            16,
            3,
            3,
        )
        .with_logical_transfer((LogicalData::EmptySequence(0..1), "101"))?;

        assert_eq!(physical_transfer.holds_valid(), false);
        assert_eq!(physical_transfer.data(), &Some(vec![]));
        assert_eq!(
            physical_transfer.last(),
            &LastMode::Lane(vec![Some(0..1), None, None])
        );
        assert_eq!(physical_transfer.start_index(), &Some(0));
        assert_eq!(physical_transfer.end_index(), &Some(0));
        assert_eq!(
            physical_transfer.strobe(),
            &StrobeMode::Lane(vec![false; 3])
        );
        assert_eq!(physical_transfer.user(), &Some(vec![true, false, true]));

        Ok(())
    }
}
