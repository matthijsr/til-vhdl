use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParamValue {
    Integer(i32),
}

impl fmt::Display for GenericParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericParamValue::Integer(val) => write!(f, "Integer({})", val),
        }
    }
}

impl From<i32> for GenericParamValue {
    fn from(val: i32) -> Self {
        GenericParamValue::Integer(val)
    }
}
