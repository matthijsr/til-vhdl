use tydi_common::error::{Result, TryResult};

use self::dimensionality::DimensionalityGeneric;

use super::{condition::TestValue, param_value::GenericParamValue};

pub mod dimensionality;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceGenericKind {
    Dimensionality(DimensionalityGeneric),
}

impl From<DimensionalityGeneric> for InterfaceGenericKind {
    fn from(val: DimensionalityGeneric) -> Self {
        Self::Dimensionality(val)
    }
}

impl TestValue for InterfaceGenericKind {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            InterfaceGenericKind::Dimensionality(dim) => dim.test_value(value),
        }
    }
}
