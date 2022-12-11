use std::{any::type_name, fmt, str::FromStr};
use tydi_common::error::{Error, Result, TryResult};

use super::param_value::GenericParamValue;

pub mod integer_condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericCondition<T: TestValue> {
    None,
    Single(T),
    Parentheses(Box<Self>),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl<T: TestValue> GenericCondition<T> {
    // This doesn't implement the trait, to avoid strange nesting behavior.
    pub fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let value = value.try_result()?;
        match self {
            GenericCondition::None => Ok(true),
            GenericCondition::Single(t) => t.test_value(value),
            GenericCondition::Parentheses(s) => s.test_value(value),
            GenericCondition::Not(n) => Ok(!n.test_value(value)?),
            GenericCondition::And(l, r) => Ok(l.test_value(value.clone())? && r.test_value(value)?),
            GenericCondition::Or(l, r) => Ok(l.test_value(value.clone())? || r.test_value(value)?),
        }
    }
}

impl<T: TestValue> From<T> for GenericCondition<T> {
    fn from(val: T) -> Self {
        Self::Single(val)
    }
}

pub trait TestValue {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool>;
}

pub trait AppliesCondition<T: TestValue>: Sized {
    fn condition(&self) -> &GenericCondition<T>;

    fn set_condition(&mut self, condition: impl TryResult<GenericCondition<T>>) -> Result<()>;

    fn with_condition(mut self, condition: impl TryResult<GenericCondition<T>>) -> Result<Self> {
        self.set_condition(condition)?;
        Ok(self)
    }
}
