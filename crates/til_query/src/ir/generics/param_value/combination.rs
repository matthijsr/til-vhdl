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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl fmt::Display for MathOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathOperator::Add => write!(f, "+"),
            MathOperator::Subtract => write!(f, "-"),
            MathOperator::Multiply => write!(f, "*"),
            MathOperator::Divide => write!(f, "/"),
            MathOperator::Modulo => write!(f, "%"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathCombination {
    Parentheses(Box<MathCombination>),
    Negative(Box<GenericParamValue>),
    Combination(Box<GenericParamValue>, MathOperator, Box<GenericParamValue>),
}

impl fmt::Display for MathCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathCombination::Parentheses(p) => write!(f, "Parentheses({})", p),
            MathCombination::Negative(n) => write!(f, "Negative({})", n),
            MathCombination::Combination(l, op, r) => write!(f, "{} {} {}", l, op, r),
        }
    }
}

impl MathCombination {
    // Performed at the end, if there are any (pointless) parentheses remaining, this will remove them
    pub fn remove_outer_parens(self) -> GenericParamValue {
        match self {
            MathCombination::Parentheses(p) => p.clone().remove_outer_parens(),
            MathCombination::Negative(_) => self.into(),
            MathCombination::Combination(_, _, _) => self.into(),
        }
    }

    pub fn verify_integer(self) -> Result<Self> {
        fn fail_if_not_int(val: &GenericParamValue) -> Result<()> {
            if val.is_integer() {
                Ok(())
            } else {
                Err(Error::InvalidArgument(format!("{} is not an integer", val)))
            }
        }

        match &self {
            MathCombination::Parentheses(_) => (),
            MathCombination::Negative(n) => fail_if_not_int(n.as_ref())?,
            MathCombination::Combination(l, _, r) => {
                fail_if_not_int(l.as_ref())?;
                fail_if_not_int(r.as_ref())?
            }
        };

        Ok(self)
    }

    fn precedence(
        l: Box<GenericParamValue>,
        op: MathOperator,
    ) -> (
        Option<(Box<GenericParamValue>, MathOperator)>,
        Box<GenericParamValue>,
    ) {
        match op {
            MathOperator::Add | MathOperator::Subtract => (None, l),
            MathOperator::Multiply | MathOperator::Divide | MathOperator::Modulo => {
                match l.as_ref() {
                    GenericParamValue::Integer(_) => (None, l),
                    GenericParamValue::Ref(_) => (None, l),
                    GenericParamValue::Combination(c) => match c {
                        Combination::Math(m) => match m {
                            MathCombination::Parentheses(_) => (None, l),
                            MathCombination::Negative(_) => (None, l),
                            MathCombination::Combination(f_l, f_op, r) => match f_op {
                                MathOperator::Add | MathOperator::Subtract => {
                                    (Some((f_l.clone(), *f_op)), r.clone())
                                }
                                MathOperator::Multiply
                                | MathOperator::Divide
                                | MathOperator::Modulo => (None, l),
                            },
                        },
                    },
                }
            }
        }
    }

    pub fn reduce(&self) -> GenericParamValue {
        match self {
            MathCombination::Parentheses(p) => match p.reduce() {
                GenericParamValue::Integer(i) => GenericParamValue::Integer(i),
                GenericParamValue::Ref(r) => GenericParamValue::Ref(r),
                GenericParamValue::Combination(c) => match c {
                    Combination::Math(m) => match m {
                        // If the inside is also Parentheses, remove the outer Parentheses
                        // Since the reduce will have recursively stripped them, this should be the last
                        MathCombination::Parentheses(inner_p) => {
                            MathCombination::Parentheses(inner_p).into()
                        }
                        // Negative effectively replaces Parentheses
                        MathCombination::Negative(n) => MathCombination::Negative(n).into(),
                        // Once we hit a combination that couldn't be reduced to a single value
                        MathCombination::Combination(l, op, r) => {
                            Self::parentheses(MathCombination::Combination(l, op, r)).into()
                        }
                    },
                },
            },
            MathCombination::Negative(n) => match n.as_ref() {
                GenericParamValue::Integer(i) => GenericParamValue::Integer(-i),
                GenericParamValue::Ref(_) => self.clone().into(),
                GenericParamValue::Combination(c) => match c {
                    Combination::Math(m) => match m {
                        MathCombination::Parentheses(p) => {
                            // Negative implicitly includes parentheses
                            MathCombination::Negative(Box::new(p.reduce())).into()
                        }
                        // Two negatives cancel out
                        MathCombination::Negative(n) => n.reduce(),
                        _ => MathCombination::Negative(Box::new(m.reduce())).into(),
                    },
                },
            },
            MathCombination::Combination(l, op, r) => {
                let (p, l) = Self::precedence(l.clone(), *op);
                // Test for precedence and re-order combination if needed
                if let Some((f_l, f_op)) = p {
                    return MathCombination::Combination(
                        f_l,
                        f_op,
                        Box::new(MathCombination::Combination(l.clone(), *op, r.clone()).reduce()),
                    )
                    .reduce();
                }
                let mut l = l.reduce();
                let mut r = r.reduce();

                if l == 0 && r == 0 {
                    // TODO: Technically incorrect for division and mod, I think?
                    return GenericParamValue::Integer(0);
                }
                if let (GenericParamValue::Integer(l), GenericParamValue::Integer(r)) = (&l, &r) {
                    return match op {
                        MathOperator::Add => GenericParamValue::Integer(l + r),
                        MathOperator::Subtract => GenericParamValue::Integer(l - r),
                        MathOperator::Multiply => GenericParamValue::Integer(l * r),
                        MathOperator::Divide => GenericParamValue::Integer(l / r),
                        MathOperator::Modulo => GenericParamValue::Integer(l % r),
                    };
                }
                if let GenericParamValue::Integer(_) = &l {
                    match op {
                        // Try to push any integers to the right, to simplify further reduction
                        MathOperator::Add | MathOperator::Multiply => (l, r) = (r, l),
                        MathOperator::Subtract => (),
                        MathOperator::Divide => (),
                        MathOperator::Modulo => (),
                    }
                }
                match op {
                    MathOperator::Add if l == r => MathCombination::Combination(
                        Box::new(GenericParamValue::Integer(2)),
                        *op,
                        Box::new(r),
                    )
                    .into(),
                    MathOperator::Add if l == 0 => r,
                    MathOperator::Subtract if l == r => GenericParamValue::Integer(0),
                    MathOperator::Subtract if l == 0 => {
                        MathCombination::Negative(Box::new(r)).into()
                    }
                    MathOperator::Add | MathOperator::Subtract => {
                        if r == 0 {
                            l
                        } else {
                            MathCombination::Combination(Box::new(l), *op, Box::new(r)).into()
                        }
                    }
                    MathOperator::Multiply => {
                        if l == 0 {
                            GenericParamValue::Integer(0)
                        } else if r == 0 {
                            GenericParamValue::Integer(0)
                        } else if l == 1 {
                            r
                        } else if r == 1 {
                            l
                        } else {
                            MathCombination::Combination(Box::new(l), *op, Box::new(r)).into()
                        }
                    }
                    MathOperator::Divide => {
                        if l == 0 {
                            GenericParamValue::Integer(0)
                        } else if r == 0 {
                            // TODO: Should throw an error here
                            todo!()
                        } else if r == 1 {
                            l
                        } else if l == r {
                            GenericParamValue::Integer(1)
                        } else {
                            MathCombination::Combination(Box::new(l), *op, Box::new(r)).into()
                        }
                    }
                    MathOperator::Modulo => {
                        if l == 0 {
                            GenericParamValue::Integer(0)
                        } else if r == 0 {
                            // TODO: Should throw an error here
                            todo!()
                        } else if r == 1 {
                            GenericParamValue::Integer(0)
                        } else if l == r {
                            GenericParamValue::Integer(0)
                        } else {
                            MathCombination::Combination(Box::new(l), *op, Box::new(r)).into()
                        }
                    }
                }
            }
        }
    }

    pub fn left_val(&self) -> &GenericParamValue {
        match self {
            MathCombination::Parentheses(p) => p.left_val(),
            MathCombination::Negative(n) => n.as_ref(),
            MathCombination::Combination(l, _, _) => l.as_ref(),
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
        Ok(MathCombination::Combination(
            Self::integer_or_err(left)?,
            MathOperator::Add,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn subtraction(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Combination(
            Self::integer_or_err(left)?,
            MathOperator::Subtract,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn product(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Combination(
            Self::integer_or_err(left)?,
            MathOperator::Multiply,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn division(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Combination(
            Self::integer_or_err(left)?,
            MathOperator::Divide,
            Self::integer_or_err(right)?,
        ))
    }

    pub fn modulo(
        left: impl Into<GenericParamValue>,
        right: impl Into<GenericParamValue>,
    ) -> Result<MathCombination> {
        Ok(MathCombination::Combination(
            Self::integer_or_err(left)?,
            MathOperator::Modulo,
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
