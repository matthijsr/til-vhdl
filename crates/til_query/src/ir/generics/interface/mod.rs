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
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            InterfaceGenericKind::Dimensionality(dim) => dim.valid_value(value),
        }
    }

    fn describe_condition(&self) -> String {
        match self {
            InterfaceGenericKind::Dimensionality(dim) => dim.describe_condition(),
        }
    }
}
