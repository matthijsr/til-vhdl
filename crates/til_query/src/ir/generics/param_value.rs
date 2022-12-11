use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericParamValue {
    Integer(i32),
    // Todo: Other kinds of values (specifically combinations and refs/ids)
    Ref,
    Combination,
}

impl fmt::Display for GenericParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericParamValue::Integer(val) => write!(f, "Integer({})", val),
            GenericParamValue::Ref => todo!(),
            GenericParamValue::Combination => todo!(),
        }
    }
}

impl From<i32> for GenericParamValue {
    fn from(val: i32) -> Self {
        GenericParamValue::Integer(val)
    }
}
