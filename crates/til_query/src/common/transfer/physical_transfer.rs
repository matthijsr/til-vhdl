use std::ops::Range;

use tydi_common::numbers::{NonNegative, Positive};

use crate::common::physical::complexity::Complexity;

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
    /// This stream has Complexity < 8, so asserts `strobe` per transfer.
    Transfer(bool),
    /// This stream has Complexity >= 8, so can assert `strobe` per element lane.
    Lane(Vec<bool>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A physical transfer for a given physical stream. Validates and organizes
/// logical transfers based on the physical stream's properties.
pub struct PhysicalTransfer {
    /// The Complexity this transfer adheres to. This value informs whether the
    /// logical transfer is valid.
    complexity: Complexity,
    /// This transfer is of an empty sequence.
    ///
    /// When C < 4, transfers of non-empty sequences may not postpone `last`
    /// signals. Meaning that they cannot assert a transfer being the `last`
    /// within dimension 0 without containing data.
    is_empty_sequence: bool,
    /// Indicates whether this transfer allows for `valid` to be released.
    /// This depends on the Complexity of the stream.
    ///
    /// * C < 3: `valid` can only be released when lane N-1 has a non-zero `last`.
    /// * C < 2: `valid` can only be released when lane N-1 has a `last` of all 1s.
    holds_valid: bool,
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

        Self {
            complexity,
            is_empty_sequence: false,
            holds_valid: false,
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

    /// The Complexity this transfer adheres to. This value informs whether the
    /// logical transfer is valid.
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }

    /// This transfer is of an empty sequence.
    ///
    /// When C < 4, transfers of non-empty sequences may not postpone `last`
    /// signals. Meaning that they cannot assert a transfer being the `last`
    /// within dimension 0 without containing data.
    pub fn is_empty_sequence(&self) -> bool {
        self.is_empty_sequence
    }

    /// Indicates whether this transfer allows for `valid` to be released.
    /// This depends on the Complexity of the stream.
    ///
    /// * C < 3: `valid` can only be released when lane N-1 has a non-zero `last`.
    /// * C < 2: `valid` can only be released when lane N-1 has a `last` of all 1s.
    pub fn holds_valid(&self) -> bool {
        self.holds_valid
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
