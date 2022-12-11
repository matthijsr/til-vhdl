use self::integer::{IntegerGeneric, IntegerGenericKind};
use tydi_common::error::{Result, TryResult};

use super::{condition::TestValue, param_value::GenericParamValue};

pub mod integer;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehavioralGenericKind {
    Integer(IntegerGeneric),
}

impl From<IntegerGeneric> for BehavioralGenericKind {
    fn from(val: IntegerGeneric) -> Self {
        Self::Integer(val)
    }
}

impl TestValue for BehavioralGenericKind {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            BehavioralGenericKind::Integer(integer) => integer.test_value(value),
        }
    }
}
