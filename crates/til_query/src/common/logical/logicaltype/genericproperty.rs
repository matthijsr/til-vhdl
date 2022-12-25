use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Rem;
use std::ops::Sub;
use tydi_common::error::TryResult;
use tydi_common::error::{Error, Result};
use tydi_common::name::Name;
use tydi_common::name::NameSelf;
use tydi_common::numbers::i32_to_u32;
use tydi_common::numbers::NonNegative;

use core::fmt;

use crate::ir::generics::param_value::combination::Combination;
use crate::ir::generics::param_value::combination::MathCombination;
use crate::ir::generics::param_value::combination::MathOperator;
use crate::ir::generics::param_value::GenericParamValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericProperty<T: fmt::Display> {
    Combination(
        Box<GenericProperty<T>>,
        MathOperator,
        Box<GenericProperty<T>>,
    ),
    Fixed(T),
    Parameterized(Name),
}

impl GenericProperty<NonNegative> {
    pub fn try_assign(
        &self,
        param: &Name,
        val: impl TryResult<GenericProperty<NonNegative>>,
    ) -> Result<Self> {
        let param = param.try_result()?;
        let val = val.try_result()?;
        Ok(match &self {
            GenericProperty::Combination(l, op, r) => GenericProperty::Combination(
                Box::new(l.try_assign(param, val.clone())?),
                *op,
                Box::new(r.try_assign(param, val.clone())?),
            ),
            GenericProperty::Fixed(_) => self.clone(),
            GenericProperty::Parameterized(n) => {
                if n == param {
                    val
                } else {
                    self.clone()
                }
            }
        })
    }

    pub fn try_eval(&self) -> Option<NonNegative> {
        match self {
            GenericProperty::Combination(l, op, r) => {
                if let Some(lv) = l.try_eval() {
                    if let Some(rv) = r.try_eval() {
                        return match op {
                            MathOperator::Add => Some(lv + rv),
                            MathOperator::Subtract => Some(lv - rv),
                            MathOperator::Multiply => Some(lv * rv),
                            MathOperator::Divide => Some(lv / rv),
                            MathOperator::Modulo => Some(lv % rv),
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
                    MathOperator::Add => {
                        if l.is_zero() {
                            r
                        } else if r.is_zero() {
                            l
                        } else {
                            l + r
                        }
                    }
                    MathOperator::Subtract => {
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
                    MathOperator::Multiply => {
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
                    MathOperator::Divide => {
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
                    MathOperator::Modulo => {
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
        GenericProperty::Combination(Box::new(self), MathOperator::Add, Box::new(rhs))
    }
}

impl<T: fmt::Display> Sub for GenericProperty<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(Box::new(self), MathOperator::Subtract, Box::new(rhs))
    }
}

impl<T: fmt::Display> Mul for GenericProperty<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(Box::new(self), MathOperator::Multiply, Box::new(rhs))
    }
}

impl<T: fmt::Display> Div for GenericProperty<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(Box::new(self), MathOperator::Divide, Box::new(rhs))
    }
}

impl<T: fmt::Display> Rem for GenericProperty<T> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        GenericProperty::Combination(Box::new(self), MathOperator::Modulo, Box::new(rhs))
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

impl TryFrom<MathCombination> for GenericProperty<NonNegative> {
    type Error = Error;

    fn try_from(value: MathCombination) -> Result<Self> {
        match value {
            MathCombination::Parentheses(p) => p.as_ref().clone().try_into(),
            MathCombination::Negative(_) => Err(Error::InvalidArgument(
                "NonNegative GenericProperty should not feature a negative operator".to_string(),
            )),
            MathCombination::Combination(l, op, r) => Ok(GenericProperty::Combination(
                Box::new(l.as_ref().clone().try_into()?),
                op,
                Box::new(r.as_ref().clone().try_into()?),
            )),
        }
    }
}

impl TryFrom<GenericParamValue> for GenericProperty<NonNegative> {
    type Error = Error;

    fn try_from(value: GenericParamValue) -> Result<Self> {
        if !value.is_integer() {
            return Err(Error::InvalidArgument(format!(
                "Cannot convert a {} into a NonNegative GenericProperty",
                value
            )));
        }
        Ok(match value {
            GenericParamValue::Integer(i) => GenericProperty::Fixed(i32_to_u32(i)?),
            GenericParamValue::Ref(r) => GenericProperty::Parameterized(r.name().clone()),
            GenericParamValue::Combination(c) => match c {
                Combination::Math(m) => m.try_into()?,
            },
        })
    }
}
