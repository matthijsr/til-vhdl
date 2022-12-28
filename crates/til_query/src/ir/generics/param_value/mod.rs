use core::fmt;

use tydi_common::error::{Error, Result};

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
    pub fn try_add_parens(self) -> Result<Self> {
        match self {
            GenericParamValue::Integer(_) | GenericParamValue::Ref(_) => {
                Err(Error::InvalidArgument(format!(
                    "Single values should not be enclosed by parentheses. {} is not suitable.",
                    self
                )))
            }
            GenericParamValue::Combination(c) => match c {
                Combination::Math(m) => Ok(MathCombination::parentheses(m).into()),
            },
        }
    }

    // Performed at the end, if there are any (pointless) parentheses remaining, this will remove them
    pub fn remove_outer_parens(self) -> Self {
        match self {
            GenericParamValue::Integer(_) => self,
            GenericParamValue::Ref(_) => self,
            GenericParamValue::Combination(c) => match c {
                Combination::Math(m) => m.remove_outer_parens(),
            },
        }
    }

    pub fn reduce(&self) -> Self {
        match self {
            GenericParamValue::Integer(_) => self.clone(),
            GenericParamValue::Ref(_) => self.clone(),
            GenericParamValue::Combination(m) => match m {
                Combination::Math(m) => m.reduce(),
            },
        }
    }

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

    pub fn is_zero(&self) -> bool {
        matches!(self, GenericParamValue::Integer(0))
    }

    pub fn is_one(&self) -> bool {
        matches!(self, GenericParamValue::Integer(0))
    }
}

impl PartialEq<i32> for GenericParamValue {
    fn eq(&self, other: &i32) -> bool {
        if let GenericParamValue::Integer(i) = self {
            i == other
        } else {
            false
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
