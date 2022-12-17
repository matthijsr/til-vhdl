use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use tydi_common::error::{Error, Result};
use tydi_common::name::Name;
use tydi_common::numbers::NonNegative;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericPropertyOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for GenericPropertyOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericPropertyOperator::Add => write!(f, "+"),
            GenericPropertyOperator::Subtract => write!(f, "-"),
            GenericPropertyOperator::Multiply => write!(f, "*"),
            GenericPropertyOperator::Divide => write!(f, "/"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericProperty<T: fmt::Display> {
    Combination(
        Box<GenericProperty<T>>,
        GenericPropertyOperator,
        Box<GenericProperty<T>>,
    ),
    Fixed(T),
    Parameterized(Name),
}

impl<
        T: fmt::Display + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy,
    > GenericProperty<T>
{
    pub fn try_eval(&self) -> Option<T> {
        match self {
            GenericProperty::Combination(l, op, r) => {
                if let Some(lv) = l.try_eval() {
                    if let Some(rv) = r.try_eval() {
                        return match op {
                            GenericPropertyOperator::Add => Some(lv + rv),
                            GenericPropertyOperator::Subtract => Some(lv - rv),
                            GenericPropertyOperator::Multiply => Some(lv * rv),
                            GenericPropertyOperator::Divide => Some(lv / rv),
                        };
                    }
                }
                None
            }
            GenericProperty::Fixed(f) => Some(*f),
            GenericProperty::Parameterized(_) => None,
        }
    }
}

impl<T: fmt::Display> fmt::Display for GenericProperty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericProperty::Combination(l, op, r) => write!(f, "({}) {} {}", l, op, r),
            GenericProperty::Fixed(val) => write!(f, "Fixed({})", val),
            GenericProperty::Parameterized(p) => write!(f, "Parameterized({})", p),
        }
    }
}

impl<T: fmt::Display> Add for GenericProperty<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(Box::new(self), GenericPropertyOperator::Add, Box::new(rhs))
    }
}

impl<T: fmt::Display> Sub for GenericProperty<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(
            Box::new(self),
            GenericPropertyOperator::Subtract,
            Box::new(rhs),
        )
    }
}

impl<T: fmt::Display> Mul for GenericProperty<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(
            Box::new(self),
            GenericPropertyOperator::Multiply,
            Box::new(rhs),
        )
    }
}

impl<T: fmt::Display> Div for GenericProperty<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(
            Box::new(self),
            GenericPropertyOperator::Divide,
            Box::new(rhs),
        )
    }
}

impl From<NonNegative> for GenericProperty<NonNegative> {
    fn from(val: NonNegative) -> Self {
        Self::Fixed(val)
    }
}

impl<T: fmt::Display> From<Name> for GenericProperty<T> {
    fn from(val: Name) -> Self {
        Self::Parameterized(val)
    }
}

impl<T: fmt::Display> From<&Name> for GenericProperty<T> {
    fn from(val: &Name) -> Self {
        Self::Parameterized(val.clone())
    }
}

impl<T: fmt::Display> TryFrom<&str> for GenericProperty<T> {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(Self::Parameterized(Name::try_new(value)?))
    }
}
