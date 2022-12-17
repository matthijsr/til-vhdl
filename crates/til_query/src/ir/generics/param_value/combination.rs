use core::fmt;

use super::GenericParamValue;
use tydi_common::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Combination {
    Math(MathCombination),
}

impl fmt::Display for Combination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combination::Math(m) => write!(f, "Math({})", m),
        }
    }
}

impl Combination {
    pub fn left_val(&self) -> &GenericParamValue {
        match self {
            Combination::Math(m) => m.left_val(),
        }
    }
}

impl From<MathCombination> for Combination {
    fn from(math: MathCombination) -> Self {
        Self::Math(math)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathCombination {
    Parentheses(Box<MathCombination>),
    Negative(Box<GenericParamValue>),
    Sum(Box<GenericParamValue>, Box<GenericParamValue>),
    Subtraction(Box<GenericParamValue>, Box<GenericParamValue>),
    Product(Box<GenericParamValue>, Box<GenericParamValue>),
    Division(Box<GenericParamValue>, Box<GenericParamValue>),
    Modulo(Box<GenericParamValue>, Box<GenericParamValue>),
}

impl fmt::Display for MathCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathCombination::Parentheses(p) => write!(f, "Parentheses({})", p),
            MathCombination::Negative(n) => write!(f, "Negative({})", n),
            MathCombination::Sum(l, r) => write!(f, "Sum({}, {})", l, r),
            MathCombination::Subtraction(l, r) => write!(f, "Subtraction({}, {})", l, r),
            MathCombination::Product(l, r) => write!(f, "Product({}, {})", l, r),
            MathCombination::Division(l, r) => write!(f, "Division({}, {})", l, r),
            MathCombination::Modulo(l, r) => write!(f, "Modulo({}, {})", l, r),
        }
    }
}

impl MathCombination {
    pub fn left_val(&self) -> &GenericParamValue {
        match self {
            MathCombination::Parentheses(p) => p.left_val(),
            MathCombination::Negative(n) => n.as_ref(),
            MathCombination::Sum(l, _)
            | MathCombination::Subtraction(l, _)
            | MathCombination::Product(l, _)
            | MathCombination::Division(l, _)
            | MathCombination::Modulo(l, _) => l.as_ref(),
        }
    }

    fn integer_or_err(val: impl Into<GenericParamValue>) -> Result<Box<GenericParamValue>> {
        let val = val.into();
        if val.is_integer() {
            Ok(Box::new(val))
        } else {
            Err(Error::InvalidArgument(format!(
                "Cannot create MathCombination with value: {}",
                val
            )))
        }
    }

    pub fn parentheses(val: impl Into<MathCombination>) -> MathCombination {
        let val = val.into();
        Self::Parentheses(Box::new(val))
    }

    pub fn negative(val: impl Into<GenericParamValue>) -> Result<MathCombination> {
        let val = val.into();
        Ok(MathCombination::Negative(Self::integer_or_err(val)?))
    }

    pub fn sum(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Sum(
            Self::integer_or_err(left)?,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn subtraction(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Subtraction(
            Self::integer_or_err(left)?,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn product(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Product(
            Self::integer_or_err(left)?,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn division(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Division(
            Self::integer_or_err(left)?,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn modulo(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Modulo(
            Self::integer_or_err(left)?,
            Self::integer_or_err(right)?,
        ))
    }
}

pub trait GenericParamValueOps {
    fn g_negative(self) -> Result<Combination>;
    fn g_add(self, right: impl Into<GenericParamValue>) -> Result<Combination>;
    fn g_sub(self, right: impl Into<GenericParamValue>) -> Result<Combination>;
    fn g_mul(self, right: impl Into<GenericParamValue>) -> Result<Combination>;
    fn g_div(self, right: impl Into<GenericParamValue>) -> Result<Combination>;
    fn g_mod(self, right: impl Into<GenericParamValue>) -> Result<Combination>;
}

impl<I: Into<GenericParamValue>> GenericParamValueOps for I {
    fn g_negative(self) -> Result<Combination> {
        Ok(MathCombination::negative(self)?.into())
    }

    fn g_add(self, right: impl Into<GenericParamValue>) -> Result<Combination> {
        Ok(MathCombination::sum(self, right)?.into())
    }

    fn g_sub(self, right: impl Into<GenericParamValue>) -> Result<Combination> {
        Ok(MathCombination::subtraction(self, right)?.into())
    }

    fn g_mul(self, right: impl Into<GenericParamValue>) -> Result<Combination> {
        Ok(MathCombination::product(self, right)?.into())
    }

    fn g_div(self, right: impl Into<GenericParamValue>) -> Result<Combination> {
        Ok(MathCombination::division(self, right)?.into())
    }

    fn g_mod(self, right: impl Into<GenericParamValue>) -> Result<Combination> {
        Ok(MathCombination::modulo(self, right)?.into())
    }
}
