use tydi_common::error::{Error, Result, TryResult};

use crate::ir::generics::{
    condition::{
        integer_condition::IntegerCondition, AppliesCondition, GenericCondition, TestValue,
    },
    param_value::GenericParamValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerGenericKind {
    Integer,
    Natural,
    Positive,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerGeneric {
    kind: IntegerGenericKind,
    condition: GenericCondition<IntegerCondition>,
}

impl IntegerGeneric {
    pub fn integer() -> Self {
        Self {
            kind: IntegerGenericKind::Integer,
            condition: GenericCondition::None,
        }
    }

    pub fn natural() -> Self {
        Self {
            kind: IntegerGenericKind::Natural,
            condition: GenericCondition::None,
        }
    }

    pub fn positive() -> Self {
        Self {
            kind: IntegerGenericKind::Positive,
            condition: GenericCondition::None,
        }
    }

    pub fn kind(&self) -> &IntegerGenericKind {
        &self.kind
    }
}

impl AppliesCondition<IntegerCondition> for IntegerGeneric {
    fn condition(&self) -> &GenericCondition<IntegerCondition> {
        &self.condition
    }

    fn set_condition(
        &mut self,
        condition: impl TryResult<GenericCondition<IntegerCondition>>,
    ) -> Result<()> {
        self.condition = condition.try_result()?;
        Ok(())
    }
}

impl TestValue for IntegerGeneric {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let generic_value: GenericParamValue = value.try_result()?;
        let value = match generic_value {
            GenericParamValue::Integer(val) => Ok(val),
            _ => Err(Error::InvalidArgument(format!(
                "Expected an Integer value, got a {}",
                generic_value
            ))),
        }?;
        match self.kind() {
            IntegerGenericKind::Natural if value < 0 => Ok(false),
            IntegerGenericKind::Positive if value < 1 => Ok(false),
            _ => self.condition().test_value(value),
        }
    }
}
