use std::fmt;
use tydi_common::error::{Result, TryResult};

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
    /// Verify whether this condition only permits values permitted by the other
    /// condition. (I.e., it is as or more restrictive.)
    pub fn satisfies(&self, _other: &Self) -> bool {
        // TODO
        true
    }
}

impl<T: TestValue> GenericCondition<T> {
    // This doesn't implement the trait, to avoid strange nesting behavior.
    pub fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        let value = value.try_result()?;
        match self {
            GenericCondition::None => Ok(true),
            GenericCondition::Single(t) => t.valid_value(value),
            GenericCondition::Parentheses(s) => s.valid_value(value),
            GenericCondition::Not(n) => Ok(!n.valid_value(value)?),
            GenericCondition::And(l, r) => {
                Ok(l.valid_value(value.clone())? && r.valid_value(value)?)
            }
            GenericCondition::Or(l, r) => {
                Ok(l.valid_value(value.clone())? || r.valid_value(value)?)
            }
        }
    }

    pub fn parens(val: impl Into<Self>) -> Self {
        Self::Parentheses(Box::new(val.into()))
    }

    pub fn not(val: impl Into<Self>) -> Self {
        Self::Not(Box::new(val.into()))
    }

    pub fn and(left: impl Into<Self>, right: impl Into<Self>) -> Self {
        Self::And(Box::new(left.into()), Box::new(right.into()))
    }

    pub fn or(left: impl Into<Self>, right: impl Into<Self>) -> Self {
        Self::Or(Box::new(left.into()), Box::new(right.into()))
    }
}

impl<T: TestValue> From<T> for GenericCondition<T> {
    fn from(val: T) -> Self {
        Self::Single(val)
    }
}

pub trait BuildsCondition<T: TestValue> {
    fn parens(self) -> GenericCondition<T>;
    fn invert(self) -> GenericCondition<T>;
    fn and(self, right: impl Into<GenericCondition<T>>) -> GenericCondition<T>;
    fn or(self, right: impl Into<GenericCondition<T>>) -> GenericCondition<T>;
}

impl<T: TestValue, I: Into<GenericCondition<T>>> BuildsCondition<T> for I {
    fn parens(self) -> GenericCondition<T> {
        GenericCondition::<T>::parens(self)
    }

    fn invert(self) -> GenericCondition<T> {
        GenericCondition::<T>::not(self)
    }

    fn and(self, right: impl Into<GenericCondition<T>>) -> GenericCondition<T> {
        GenericCondition::<T>::and(self, right)
    }

    fn or(self, right: impl Into<GenericCondition<T>>) -> GenericCondition<T> {
        GenericCondition::<T>::or(self, right)
    }
}

pub trait TestValue: Sized {
    fn describe_condition(&self) -> String;
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool>;
}

pub trait AppliesCondition<T: TestValue>: Sized {
    fn condition(&self) -> &GenericCondition<T>;

    fn set_condition(&mut self, condition: impl TryResult<GenericCondition<T>>) -> Result<()>;

    fn with_condition(mut self, condition: impl TryResult<GenericCondition<T>>) -> Result<Self> {
        self.set_condition(condition)?;
        Ok(self)
    }
}

impl<T: TestValue> fmt::Display for GenericCondition<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericCondition::None => write!(f, "None"),
            GenericCondition::Single(t) => write!(f, "{}", t.describe_condition()),
            GenericCondition::Parentheses(s) => write!(f, "({})", s),
            GenericCondition::Not(n) => write!(f, "!({})", n),
            GenericCondition::And(l, r) => write!(f, "{} and {}", l, r),
            GenericCondition::Or(l, r) => write!(f, "{} or {}", l, r),
        }
    }
}
