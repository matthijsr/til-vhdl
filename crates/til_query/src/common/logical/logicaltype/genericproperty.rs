use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Rem;
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
    Modulo,
}

impl fmt::Display for GenericPropertyOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericPropertyOperator::Add => write!(f, "+"),
            GenericPropertyOperator::Subtract => write!(f, "-"),
            GenericPropertyOperator::Multiply => write!(f, "*"),
            GenericPropertyOperator::Divide => write!(f, "/"),
            GenericPropertyOperator::Modulo => write!(f, "%"),
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

impl GenericProperty<NonNegative> {
    pub fn try_eval(&self) -> Option<NonNegative> {
        match self {
            GenericProperty::Combination(l, op, r) => {
                if let Some(lv) = l.try_eval() {
                    if let Some(rv) = r.try_eval() {
                        return match op {
                            GenericPropertyOperator::Add => Some(lv + rv),
                            GenericPropertyOperator::Subtract => Some(lv - rv),
                            GenericPropertyOperator::Multiply => Some(lv * rv),
                            GenericPropertyOperator::Divide => Some(lv / rv),
                            GenericPropertyOperator::Modulo => Some(lv % rv),
                        };
                    }
                }
                None
            }
            GenericProperty::Fixed(f) => Some(*f),
            GenericProperty::Parameterized(_) => None,
        }
    }

    pub fn is_one(&self) -> bool {
        if let GenericProperty::Fixed(1) = self {
            true
        } else {
            false
        }
    }

    pub fn is_zero(&self) -> bool {
        if let GenericProperty::Fixed(0) = self {
            true
        } else {
            false
        }
    }

    /// Tries to remove unnecessary operations
    ///
    /// E.g.:
    /// * N - N = 0
    /// * N * 1 = N
    /// * N / 1 = N
    /// * N * 0 = 0
    /// * etc.
    pub fn try_reduce(&self) -> Self {
        match self {
            GenericProperty::Combination(l, op, r) => {
                let l = l.try_reduce();
                let r = r.try_reduce();
                if l.is_zero() && r.is_zero() {
                    return GenericProperty::Fixed(0);
                }
                match op {
                    GenericPropertyOperator::Add => {
                        if l.is_zero() {
                            r
                        } else if r.is_zero() {
                            l
                        } else {
                            l + r
                        }
                    }
                    GenericPropertyOperator::Subtract => {
                        if l.is_zero() {
                            r
                        } else if r.is_zero() {
                            l
                        } else if l == r {
                            GenericProperty::Fixed(0)
                        } else {
                            l - r
                        }
                    }
                    GenericPropertyOperator::Multiply => {
                        if l.is_zero() {
                            GenericProperty::Fixed(0)
                        } else if r.is_zero() {
                            GenericProperty::Fixed(0)
                        } else if l.is_one() {
                            r
                        } else if r.is_one() {
                            l
                        } else {
                            l * r
                        }
                    }
                    GenericPropertyOperator::Divide => {
                        if l.is_zero() {
                            GenericProperty::Fixed(0)
                        } else if r.is_one() {
                            l
                        } else if r.is_zero() {
                            // Might want to throw an error?
                            l / r
                        } else {
                            l / r
                        }
                    }
                    GenericPropertyOperator::Modulo => {
                        if l.is_zero() {
                            GenericProperty::Fixed(0)
                        } else if r.is_zero() {
                            // TODO: This should be an error
                            todo!()
                        } else if r.is_one() {
                            GenericProperty::Fixed(0)
                        } else {
                            l % r
                        }
                    }
                }
            }
            GenericProperty::Fixed(_) => self.clone(),
            GenericProperty::Parameterized(_) => self.clone(),
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

impl<T: fmt::Display> Rem for GenericProperty<T> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(
            Box::new(self),
            GenericPropertyOperator::Modulo,
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
