use tydi_common::error::{Error, Result, TryResult};

use crate::ir::generics::{
    condition::{
        integer_condition::IntegerCondition, AppliesCondition, GenericCondition, TestValue,
    },
    param_value::GenericParamValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionalityGeneric {
    condition: GenericCondition<IntegerCondition>,
}

impl DimensionalityGeneric {
    pub fn new() -> Self {
        Self {
            condition: GenericCondition::None,
        }
    }
}

impl AppliesCondition<IntegerCondition> for DimensionalityGeneric {
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

impl TestValue for DimensionalityGeneric {
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let generic_value: GenericParamValue = value.try_result()?;
        if let GenericParamValue::Integer(value) = generic_value {
            if value < 2 {
                Ok(false)
            } else {
                self.condition().valid_value(value)
            }
        } else if generic_value.is_integer() {
            Ok(true)
        } else {
            Err(Error::InvalidArgument(format!(
                "Expected an Integer value, got a {}",
                generic_value
            )))
        }
    }

    fn describe_condition(&self) -> String {
        let base = "(Dimensionality, implicit: >= 2)";
        if let GenericCondition::None = self.condition() {
            base.to_string()
        } else {
            format!("{} and {}", base, self.condition())
        }
    }
}
