use core::fmt;

use tydi_common::error::{Error, Result, TryResult};

use crate::ir::generics::param_value::GenericParamValue;

use super::TestValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerCondition {
    Gt(i32),
    Lt(i32),
    GtEq(i32),
    LtEq(i32),
    Eq(i32),
    IsIn(Vec<i32>),
}

impl TestValue for IntegerCondition {
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let generic_value: GenericParamValue = value.try_result()?;
        let value = match generic_value {
            GenericParamValue::Integer(val) => Ok(val),
            _ => Err(Error::InvalidArgument(format!(
                "Expected an Integer value, got a {}",
                generic_value
            ))),
        }?;
        match self {
            IntegerCondition::Gt(test) => Ok(value > *test),
            IntegerCondition::Lt(test) => Ok(value < *test),
            IntegerCondition::GtEq(test) => Ok(value >= *test),
            IntegerCondition::LtEq(test) => Ok(value <= *test),
            IntegerCondition::Eq(test) => Ok(value == *test),
            IntegerCondition::IsIn(test) => Ok(test.contains(&value)),
        }
    }

    fn describe_condition(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for IntegerCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerCondition::Gt(val) => write!(f, "> {}", val),
            IntegerCondition::Lt(val) => write!(f, "< {}", val),
            IntegerCondition::GtEq(val) => write!(f, ">= {}", val),
            IntegerCondition::LtEq(val) => write!(f, "<= {}", val),
            IntegerCondition::Eq(val) => write!(f, "== {}", val),
            IntegerCondition::IsIn(list) => write!(
                f,
                "in({})",
                list.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
