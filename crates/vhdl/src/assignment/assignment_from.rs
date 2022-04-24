use bitvec::prelude::BitVec;
use tydi_intern::Id;

use crate::{declaration::ObjectDeclaration, statement::relation::Relation};

use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecValue, Assignment, AssignmentKind,
    DirectAssignment, ObjectSelection, StdLogicValue, ValueAssignment,
};

// I feel like there should be some way for Rust to recognize these connections automatically but unfortunately we can't just string "T: Into<...>"s together,
// due to potentially conflicting trait implementations: https://users.rust-lang.org/t/conflicting-implementations-of-trait/53055

impl<T> From<T> for Assignment
where
    T: Into<AssignmentKind>,
{
    fn from(kind: T) -> Self {
        Assignment {
            kind: kind.into(),
            to_field: vec![],
        }
    }
}

impl<T: Into<Relation>> From<T> for AssignmentKind {
    fn from(relation: T) -> Self {
        AssignmentKind::Relation(relation.into())
    }
}

impl From<ArrayAssignment> for AssignmentKind {
    fn from(assignment: ArrayAssignment) -> Self {
        AssignmentKind::Direct(assignment.into())
    }
}

impl<T> From<T> for ObjectSelection
where
    T: Into<Id<ObjectDeclaration>>,
{
    fn from(object: T) -> Self {
        ObjectSelection {
            object: object.into(),
            from_field: vec![],
        }
    }
}

impl From<ArrayAssignment> for DirectAssignment {
    fn from(assignment: ArrayAssignment) -> Self {
        DirectAssignment::FullArray(assignment.into())
    }
}

impl From<StdLogicValue> for ValueAssignment {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::Bit(assignment.into())
    }
}

impl From<BitVecValue> for ValueAssignment {
    fn from(assignment: BitVecValue) -> Self {
        ValueAssignment::BitVec(assignment.into())
    }
}

impl From<BitVec> for ValueAssignment {
    fn from(bv: BitVec) -> Self {
        if bv.len() == 1 {
            ValueAssignment::Bit(StdLogicValue::Logic(bv[0]))
        } else {
            let mut result_vec = vec![];
            // As the BitVec is Lsb0, reverse it to follow Msb0 numbering instead
            for val in bv.iter().by_vals().rev() {
                result_vec.push(StdLogicValue::Logic(val))
            }
            ValueAssignment::BitVec(BitVecValue::Full(result_vec))
        }
    }
}
