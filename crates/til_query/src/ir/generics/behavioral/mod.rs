use self::integer::IntegerGeneric;
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
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            BehavioralGenericKind::Integer(integer) => integer.valid_value(value),
        }
    }

    fn describe_condition(&self) -> String {
        match self {
            BehavioralGenericKind::Integer(integer) => integer.describe_condition(),
        }
    }
}
