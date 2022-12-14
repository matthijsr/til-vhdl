use core::fmt;

use tydi_common::error::{Error, Result, TryResult};

use crate::ir::generics::{
    condition::{
        integer_condition::IntegerCondition, AppliesCondition, GenericCondition, TestValue,
    },
    interface::InterfaceGenericKind,
    param_value::GenericParamValue,
    GenericKind,
};

use super::BehavioralGenericKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerGenericKind {
    Integer,
    Natural,
    Positive,
}

impl fmt::Display for IntegerGenericKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerGenericKind::Integer => write!(f, "Integer"),
            IntegerGenericKind::Natural => write!(f, "Natural"),
            IntegerGenericKind::Positive => write!(f, "Positive"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerGeneric {
    kind: IntegerGenericKind,
    condition: GenericCondition<IntegerCondition>,
}

impl fmt::Display for IntegerGeneric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind())
    }
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
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let generic_value: GenericParamValue = value.try_result()?;
        let value = match &generic_value {
            GenericParamValue::Integer(val) => Ok(*val),
            GenericParamValue::Ref(val) => match val.kind() {
                GenericKind::Behavioral(b) => match b {
                    BehavioralGenericKind::Integer(_) => return Ok(true),
                    _ => Err(Error::InvalidArgument(format!(
                        "Expected an Integer value, got a {}",
                        generic_value
                    ))),
                },
                GenericKind::Interface(i) => match i {
                    InterfaceGenericKind::Dimensionality(_) => return Ok(true),
                },
            },
            _ => Err(Error::InvalidArgument(format!(
                "Expected an Integer value, got a {}",
                generic_value
            ))),
        }?;
        match self.kind() {
            IntegerGenericKind::Natural if value < 0 => Ok(false),
            IntegerGenericKind::Positive if value < 1 => Ok(false),
            _ => self.condition().valid_value(value),
        }
    }

    fn describe_condition(&self) -> String {
        if let GenericCondition::None = self.condition() {
            match self.kind() {
                IntegerGenericKind::Integer => "",
                IntegerGenericKind::Natural => "(Natural, implicit: >= 0) and ",
                IntegerGenericKind::Positive => "(Positive, implicit: >= 1) and ",
            }
            .to_string()
        } else {
            let implicit_condition = match self.kind() {
                IntegerGenericKind::Integer => "",
                IntegerGenericKind::Natural => "(Natural, implicit: >= 0) and ",
                IntegerGenericKind::Positive => "(Positive, implicit: >= 1) and ",
            };
            format!("{}{}", implicit_condition, self.condition())
        }
    }
}
