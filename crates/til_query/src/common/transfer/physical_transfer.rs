use core::fmt;
use std::ops::Range;

use tydi_common::{
    error::{Error, Result, TryResult},
    numbers::{NonNegative, Positive},
};

use crate::common::physical::complexity::Complexity;

use super::{
    element_type::ElementType,
    logical_transfer::{LogicalData, LogicalTransfer},
};

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
/// Indicates whether this index signal is supported, and whether its value
/// is significant.
pub enum IndexMode {
    /// The physical stream does not support indices of this kind.
    Unsupported,
    /// The physical stream supports indices.
    ///
    /// When the index is `None`, its value is insignificant and does not need
    /// to be driven.
    ///
    /// Indices are insignificant unless all `strb` bits are driven high.
    Index(Option<NonNegative>),
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

impl LastMode {
    /// Determines whether this transfers the last of an inner sequence.
    ///
    /// Always returns `true` when LastMode is `None`
    pub fn last_inner(&self) -> bool {
        match self {
            LastMode::None => true,
            LastMode::Transfer(last_range) => {
                if let Some(last_range) = last_range {
                    last_range.start == 0
                } else {
                    false
                }
            }
            LastMode::Lane(last_lanes) => {
                if let Some(Some(last_range)) = last_lanes.last() {
                    last_range.start == 0
                } else {
                    false
                }
            }
        }
    }
}

impl fmt::Display for LastMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn opt_range_to_str(opt_range: &Option<Range<NonNegative>>) -> String {
            opt_range
                .as_ref()
                .map_or("None".to_string(), |r| format!("{}..{}", r.start, r.end))
        }

        match self {
            LastMode::None => write!(f, "None"),
            LastMode::Transfer(transfer) => write!(f, "Transfer({})", opt_range_to_str(transfer)),
            LastMode::Lane(lanes) => write!(
                f,
                "Lane({})",
                lanes
                    .iter()
                    .map(opt_range_to_str)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
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

impl StrobeMode {
    /// Indicates whether all `strb` bits should be driven high.
    ///
    /// This also indicates whether indices are significant.
    ///
    /// When StrobeMode is None, returns true.
    pub fn all_high(&self) -> bool {
        match self {
            StrobeMode::None => true,
            StrobeMode::Transfer(transfer) => *transfer,
            StrobeMode::Lane(lanes) => !lanes.iter().any(|x| !x),
        }
    }
}

impl fmt::Display for StrobeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrobeMode::None => write!(f, "None"),
            StrobeMode::Transfer(transfer) => write!(f, "Transfer({})", transfer),
            StrobeMode::Lane(lanes) => write!(
                f,
                "Lane({:?})",
                lanes
                    .iter()
                    .map(|x| if *x { '1' } else { '0' })
                    .collect::<String>()
            ),
        }
    }
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
    /// The (maximum) size of the element type being transferred.
    element_size: NonNegative,
    /// The data being transfered, organized by lane.
    ///
    /// When None, treat all lanes as active, but drive all '0's.
    ///
    /// When Some, but empty or short of the number of element lanes, do not
    /// drive the unaddressed lanes.
    ///
    /// Likewise do not drive lanes which have a None.
    ///
    /// When Some, and certain elements are shorter than the maximum element
    /// size, shift to align to the LSB and do not drive the unaddressed bits.
    data: Option<Vec<Option<ElementType>>>,
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
    start_index: IndexMode,
    /// The index of the last active lane.
    ///
    /// Requires (C≥5∨D≥1)∧N>1
    ///
    /// * May not be N or greater.
    /// * May not be less than `start_index`.
    ///
    /// When C < 5, and `last` is zero, end index must be N-1.
    end_index: IndexMode,
    /// The `strb` signal.
    ///
    /// Requires: C≥7∨D≥1
    ///
    /// At C < 7, this is used to indicate whether the transfer is empty.
    /// At C >= 7, this indicates the activity of individual element lanes.
    strobe: StrobeMode,
    /// The size of the `user` element type.
    user_size: NonNegative,
    /// The `user` signal
    ///
    /// When None, drive all '0's.
    ///
    /// When Some(None), do not drive.
    user: Option<Option<ElementType>>,
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
            IndexMode::Index(Some(0))
        } else {
            IndexMode::Unsupported
        };

        let end_index = if element_lanes_gt_1
            && (dimensionality >= 1 || complexity >= Complexity::new_major(5))
        {
            IndexMode::Index(Some(element_lanes.get() - 1))
        } else {
            IndexMode::Unsupported
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
            element_size: max_element_size,
            data: None,
            dimensionality,
            last: last_mode,
            start_index,
            end_index,
            strobe,
            user_size: max_user_size,
            user: None,
        }
    }

    pub fn with_logical_transfer(
        mut self,
        transfer: impl TryResult<LogicalTransfer>,
    ) -> Result<Self> {
        let transfer: LogicalTransfer = transfer.try_result()?;

        // TODO: See if it isn't possible to reuse some logic between empty sequences and logical data.
        // As it is, this might introduce some bugs by overlooking rules in one or the other.
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

                if let IndexMode::Index(stai) = &mut self.start_index {
                    *stai = None;
                }

                if let IndexMode::Index(endi) = &mut self.end_index {
                    *endi = None;
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

                let comp_lt_4 = self.complexity() < &Complexity::new_major(4);

                let mut transfer_last: Result<Option<Range<NonNegative>>> = Ok(None);

                let mut pos_edge = 0;
                let mut prev_neg = true;
                let mut transfer_strobe = false;

                let mut strobe: Vec<bool> = vec![];

                let mut start_index: Option<usize> = None;
                let mut end_index: usize = 0;

                let mut errs: Vec<String> = vec![];

                let (data_vec, last_vec): (Vec<_>, Vec<_>) = elements
                    .iter()
                    .enumerate()
                    .map(|(idx, element)| {
                        let data = match element.data() {
                            Some(data) => {
                                if data.len() != self.element_size().try_into().unwrap() {
                                    errs.push(format!("Logical transfer contains an element with size {}, which does not match the expected element size {}.", data.len(), self.element_size()));
                                }

                                if let Ok(Some(_)) = &transfer_last {
                                    transfer_last = Err(Error::InvalidArgument("Logical transfer contains an element with active data after an element was asserted last in a sequence.\n\
                                    The physical stream only supports dimension information per transfer.".to_string()));
                                }

                                if start_index.is_none() {
                                    start_index = Some(idx);
                                }
                                end_index = idx;

                                if prev_neg {
                                    pos_edge += 1;
                                    prev_neg = false;
                                }

                                transfer_strobe = true;
                                strobe.push(true);

                                Some(data.clone())
                            },
                            None => {
                                prev_neg = true;

                                strobe.push(false);

                                None
                            },
                        };

                        match element.last() {
                            Some(last_range) => {
                                if last_range.end >= self.dimensionality() {
                                    errs.push(format!("Cannot assert an element or transfer as last in dimension {}, physical stream has dimensionality {}.", last_range.end, self.dimensionality));
                                }

                                if element.data().is_none() && comp_lt_4 {
                                    transfer_last = Err(Error::InvalidArgument(format!("Cannot assert dimensionality on inactive data (cannot postpone last signals).\n\
                                    Physical stream has complexity {} (< 4).\n\
                                    If this is an empty sequence, use the `EmptySequence` LogicalTransfer.", self.complexity())));
                                }

                                match transfer_last {
                                    Ok(None) => {
                                        transfer_last = Ok(Some(last_range.clone()));
                                    },
                                    Ok(Some(_)) => {
                                        transfer_last = Err(Error::InvalidArgument(format!("Cannot assert dimensionality on more than one element lane. Physical stream has complexity {} (< 8).", self.complexity())))
                                    },
                                    Err(_) => (),
                                }
                            },
                            None => (),
                        }

                        (data, element.last().clone())
                    })
                    .unzip();

                if errs.len() > 0 {
                    return Err(Error::InvalidArgument(format!(
                        "One or more errors in logical transfer:\n{}",
                        errs.join("\n")
                    )));
                }

                match &mut self.last {
                    LastMode::None => (),
                    LastMode::Transfer(mut_transfer_last) => {
                        *mut_transfer_last = transfer_last?;

                        match &mut self.holds_valid {
                            HoldValidRule::WholeSequence(holds_valid) => {
                                if let Some(result_last) = &mut_transfer_last {
                                    *holds_valid = result_last.end == self.dimensionality - 1;
                                } else {
                                    *holds_valid = true;
                                }
                            }
                            HoldValidRule::InnerSequence(holds_valid) => {
                                *holds_valid = mut_transfer_last.is_none();
                            }
                            HoldValidRule::None => (),
                        }
                    }
                    LastMode::Lane(mut_last_vec) => {
                        *mut_last_vec = last_vec;
                        // Pad the difference with `None`s
                        for _ in mut_last_vec.len()..self.element_lanes.get().try_into().unwrap() {
                            mut_last_vec.push(None);
                        }
                    }
                }

                match &mut self.strobe {
                    StrobeMode::None => {}
                    StrobeMode::Transfer(mut_strb) => {
                        if pos_edge < 2 {
                            *mut_strb = transfer_strobe;
                        } else {
                            return Err(Error::InvalidArgument(
                                "Physical stream does not support per-lane strobed data validity."
                                    .to_string(),
                            ));
                        }
                    }
                    StrobeMode::Lane(mut_strb) => {
                        *mut_strb = strobe;
                        // Pad the difference with `false`s
                        for _ in mut_strb.len()..self.element_lanes.get().try_into().unwrap() {
                            mut_strb.push(false);
                        }
                    }
                }

                match &mut self.start_index {
                    IndexMode::Unsupported => {
                        if let Some(stai) = start_index {
                            if stai > 0 {
                                return Err(Error::InvalidArgument(format!("The physical stream requires that all transfers are aligned to lane 0, logical transfer has start index {}", stai)));
                            }
                        }
                    }
                    IndexMode::Index(mut_stai) => {
                        if self.strobe.all_high() {
                            *mut_stai = start_index.map(|x| x.try_into().unwrap())
                        } else {
                            *mut_stai = None
                        }
                    }
                }

                match &mut self.end_index {
                    IndexMode::Unsupported => {
                        // NOTE: Wait, this seems odd? Is a Stream with dimensionality 0 not allowed to have an end index when N > 1?
                        if end_index > 0 {
                            return Err(Error::InvalidArgument("This physical stream does not support end indices. (Spec issue: https://github.com/abs-tudelft/tydi/issues/226)".to_string()));
                        }
                    }
                    IndexMode::Index(mut_endi) => {
                        if self.strobe.all_high() {
                            let end_index = end_index.try_into().unwrap();
                            *mut_endi = Some(end_index);

                            if comp_lt_4
                                && end_index < self.element_lanes().get() - 1
                                && !self.last().last_inner()
                            {
                                return Err(Error::InvalidArgument(format!("Cannot leave element lanes empty, except when transferring the last element of an innermost sequence.\n\
                            Physical stream has complexity {} (< 4).", self.complexity())));
                            }
                        } else {
                            *mut_endi = None
                        }
                    }
                }

                self.data = Some(data_vec);
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

    /// The (maximum) size of the element type being transferred.
    pub fn element_size(&self) -> NonNegative {
        self.element_size
    }

    /// The data being transfered, organized by lane.
    ///
    /// When None, treat all lanes as active, but drive all '0's.
    ///
    /// When Some, but empty or short of the number of element lanes, do not
    /// drive the unaddressed lanes.
    ///
    /// Likewise do not drive lanes which have a None.
    ///
    /// When Some, and certain elements are shorter than the maximum element
    /// size, shift to align to the LSB and do not drive the unaddressed bits.
    pub fn data(&self) -> &Option<Vec<Option<ElementType>>> {
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
    pub fn start_index(&self) -> &IndexMode {
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
    pub fn end_index(&self) -> &IndexMode {
        &self.end_index
    }

    /// The `strb` signal.
    ///
    /// Requires: C≥7∨D≥1
    ///
    /// At C < 7, this is used to indicate whether the transfer is empty.
    /// At C >= 7, this indicates the activity of individual element lanes.
    pub fn strobe(&self) -> &StrobeMode {
        &self.strobe
    }

    /// The size of the `user` element type.
    pub fn user_size(&self) -> NonNegative {
        self.user_size
    }

    /// The `user` signal
    ///
    /// When None, drive all '0's.
    ///
    /// When Some(None), do not drive.
    pub fn user(&self) -> &Option<Option<ElementType>> {
        &self.user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bitvec::prelude::*;

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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Unsupported);
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(Some(2)));
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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Unsupported);
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(Some(2)));
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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(Some(0)));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(Some(2)));
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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(Some(0)));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(Some(2)));
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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(Some(0)));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(Some(2)));
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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Unsupported);
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(
            physical_transfer.user(),
            &Some(Some(ElementType::Bits(bitvec![1, 0, 1])))
        );

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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Unsupported);
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(
            physical_transfer.user(),
            &Some(Some(ElementType::Bits(bitvec![1, 0, 1])))
        );

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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.strobe(), &StrobeMode::Transfer(false));
        assert_eq!(
            physical_transfer.user(),
            &Some(Some(ElementType::Bits(bitvec![1, 0, 1])))
        );

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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(None));
        assert_eq!(
            physical_transfer.strobe(),
            &StrobeMode::Lane(vec![false; 3])
        );
        assert_eq!(
            physical_transfer.user(),
            &Some(Some(ElementType::Bits(bitvec![1, 0, 1])))
        );

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
        assert_eq!(physical_transfer.start_index(), &IndexMode::Index(None));
        assert_eq!(physical_transfer.end_index(), &IndexMode::Index(None));
        assert_eq!(
            physical_transfer.strobe(),
            &StrobeMode::Lane(vec![false; 3])
        );
        assert_eq!(
            physical_transfer.user(),
            &Some(Some(ElementType::Bits(bitvec![1, 0, 1])))
        );

        Ok(())
    }

    #[test]
    fn test_sequence_errs() -> Result<()> {
        assert_eq!(
            PhysicalTransfer::new(Complexity::new_major(1), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([("10", None), ("11", Some(0..2)), ("10", None)], "101")),
            Err(Error::InvalidArgument("Logical transfer contains an element with active data after an element was asserted last in a sequence.\nThe physical stream only supports dimension information per transfer.".to_string()))
        );

        assert_eq!(
            PhysicalTransfer::new(Complexity::new_major(1), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([("10", None), ("11", None)], "101")),
            Err(Error::InvalidArgument("Cannot leave element lanes empty, except when transferring the last element of an innermost sequence.\nPhysical stream has complexity 1 (< 4).".to_string()))
        );

        assert_eq!(
            PhysicalTransfer::new(Complexity::new_major(1), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([None, Some("11"), Some("11")], "101")),
            Err(Error::InvalidArgument("The physical stream requires that all transfers are aligned to lane 0, logical transfer has start index 1".to_string()))
        );

        assert_eq!(
            PhysicalTransfer::new(Complexity::new_major(1), Positive::new(3).unwrap(), 2, 3, 3)
                .with_logical_transfer(([Some("11"), None, Some("11")], "101")),
            Err(Error::InvalidArgument(
                "Physical stream does not support per-lane strobed data validity.".to_string()
            ))
        );

        Ok(())
    }
}
