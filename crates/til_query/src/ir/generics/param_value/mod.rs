use core::fmt;

use self::ref_value::RefValue;

pub mod ref_value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParamValue {
    Integer(i32),
    Ref(RefValue),
    // Todo: Combinations (mainly mathematical ones, for now)
    Combination,
}

impl fmt::Display for GenericParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericParamValue::Integer(val) => write!(f, "Integer({})", val),
            GenericParamValue::Ref(val) => write!(f, "Ref({})", val),
            GenericParamValue::Combination => todo!(),
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
