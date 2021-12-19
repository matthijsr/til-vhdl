use indexmap::IndexMap;
use tydi_intern::Id;

use super::{AssignmentKind, RangeConstraint};

/// An enum for describing complete assignment to an array
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArrayAssignment {
    /// Assigning all of an array directly (may concatenate objects)
    Direct(Vec<AssignmentKind>),
    /// Assign some fields directly, and may assign all other fields a single value (e.g. ( 1 => '1', others => '0' ), or ( 1 downto 0 => '1', others => '0' ))
    Sliced {
        direct: Vec<RangeAssignment>,
        others: Option<Box<AssignmentKind>>,
    },
    /// Assigning a single value to all of an array
    Others(Box<AssignmentKind>),
}

impl ArrayAssignment {
    pub fn direct(values: Vec<AssignmentKind>) -> ArrayAssignment {
        ArrayAssignment::Direct(values)
    }

    pub fn partial(
        direct: Vec<RangeAssignment>,
        others: Option<AssignmentKind>,
    ) -> ArrayAssignment {
        ArrayAssignment::Sliced {
            direct,
            others: match others {
                Some(value) => Some(Box::new(value)),
                None => None,
            },
        }
    }

    pub fn others(value: AssignmentKind) -> ArrayAssignment {
        ArrayAssignment::Others(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeAssignment {
    constraint: RangeConstraint,
    assignment: AssignmentKind,
}

impl RangeAssignment {
    pub fn new(constraint: RangeConstraint, assignment: AssignmentKind) -> Self {
        RangeAssignment {
            constraint,
            assignment,
        }
    }

    pub fn constraint(&self) -> &RangeConstraint {
        &self.constraint
    }

    pub fn assignment(&self) -> &AssignmentKind {
        &self.assignment
    }
}
