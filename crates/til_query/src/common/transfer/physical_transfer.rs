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
    data: Vec<Vec<bool>>,
    /// The dimensionality supported by the physical stream.
    dimensionality: NonNegative,
    /// The `last` signalling for the transfer.
    last_mode: LastMode,
    /// The index of the first active lane.
    ///
    /// May not be N or greater.
    ///
    /// When C < 6, must always be 0.
    start_index: NonNegative,
    /// The index of the last active lane.
    ///
    /// * May not be 0.
    /// * May not be N or greater.
    /// * May not be less than `start_index`.
    ///
    /// When C < 5, and `last` is zero, end index must be N-1.
    end_index: Positive,
    /// The `strb` signal.
    ///
    /// At C < 8, this is used to indicate whether the transfer is empty.
    /// At C >= 8, this indicates the activity of individual element lanes.
    strobe: StrobeMode,
    /// The maximum width of the `user` signal being transferred.
    max_user_size: NonNegative,
    /// The `user` signal
    user: Vec<bool>,
}

impl PhysicalTransfer {
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
    pub fn data(&self) -> &Vec<Vec<bool>> {
        &self.data
    }

    /// The dimensionality supported by the physical stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// The `last` signalling for the transfer.
    pub fn last_mode(&self) -> &LastMode {
        &self.last_mode
    }

    /// The index of the first active lane.
    ///
    /// May not be N or greater.
    ///
    /// When C < 6, must always be 0.
    pub fn start_index(&self) -> NonNegative {
        self.start_index
    }

    /// The index of the last active lane.
    ///
    /// * May not be 0.
    /// * May not be N or greater.
    /// * May not be less than `start_index`.
    ///
    /// When C < 5, and `last` is zero, end index must be N-1.
    pub fn end_index(&self) -> Positive {
        self.end_index
    }

    /// The `strb` signal.
    ///
    /// At C < 8, this is used to indicate whether the transfer is empty.
    /// At C >= 8, this indicates the activity of individual element lanes.
    pub fn strobe(&self) -> &StrobeMode {
        &self.strobe
    }

    /// The maximum width of the `user` signal being transferred.
    pub fn max_user_size(&self) -> NonNegative {
        self.max_user_size
    }

    /// The `user` signal
    pub fn user(&self) -> &Vec<bool> {
        &self.user
    }
}
