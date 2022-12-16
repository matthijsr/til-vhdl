use core::fmt;

use self::{
    combination::{Combination, MathCombination},
    ref_value::RefValue,
};

use super::{behavioral::BehavioralGenericKind, interface::InterfaceGenericKind, GenericKind};

pub mod combination;
pub mod ref_value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParamValue {
    Integer(i32),
    Ref(RefValue),
    Combination(Combination),
}

impl GenericParamValue {
    pub fn is_integer(&self) -> bool {
        match &self {
            GenericParamValue::Integer(_) => true,
            GenericParamValue::Ref(r) => match r.kind() {
                GenericKind::Behavioral(b) => match b {
                    BehavioralGenericKind::Integer(_) => true,
                },
                GenericKind::Interface(i) => match i {
                    InterfaceGenericKind::Dimensionality(_) => true,
                },
            },
            GenericParamValue::Combination(c) => c.left_val().is_integer(),
        }
    }

    pub fn is_fixed(&self) -> bool {
        match self {
            GenericParamValue::Integer(_) => true,
            GenericParamValue::Ref(_) => false,
            GenericParamValue::Combination(_) => false,
        }
    }
}

impl fmt::Display for GenericParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericParamValue::Integer(val) => write!(f, "Integer({})", val),
            GenericParamValue::Ref(val) => write!(f, "Ref({})", val),
            GenericParamValue::Combination(c) => write!(f, "Combination({})", c),
        }
    }
}

impl From<i32> for GenericParamValue {
    fn from(val: i32) -> Self {
        GenericParamValue::Integer(val)
    }
}

impl<I: Into<RefValue>> From<I> for GenericParamValue {
    fn from(i: I) -> Self {
        GenericParamValue::Ref(i.into())
    }
}

impl From<Combination> for GenericParamValue {
    fn from(combination: Combination) -> Self {
        GenericParamValue::Combination(combination)
    }
}

impl From<MathCombination> for GenericParamValue {
    fn from(math: MathCombination) -> Self {
        GenericParamValue::Combination(math.into())
    }
}
