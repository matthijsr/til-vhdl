use tydi_common::error::{Result, TryResult};

use crate::{architecture::arch_storage::Arch, declaration::DeclareWithIndent};

use super::Relation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathExpression {
    Negative(Box<Relation>),
    Sum(Box<Relation>, Box<Relation>),
    Subtraction(Box<Relation>, Box<Relation>),
    Product(Box<Relation>, Box<Relation>),
    Division(Box<Relation>, Box<Relation>),
}

impl MathExpression {
    fn validate_integer(
        db: &dyn Arch,
        relation: impl TryResult<Relation>,
    ) -> Result<Box<Relation>> {
        let relation = relation.try_result()?;
        relation.is_integer(db)?;
        Ok(Box::new(relation))
    }

    pub fn negative(db: &dyn Arch, relation: impl TryResult<Relation>) -> Result<MathExpression> {
        Ok(MathExpression::Negative(Self::validate_integer(
            db, relation,
        )?))
    }

    pub fn sum(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<MathExpression> {
        Ok(MathExpression::Sum(
            Self::validate_integer(db, left)?,
            Self::validate_integer(db, right)?,
        ))
    }

    pub fn subtraction(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<MathExpression> {
        Ok(MathExpression::Subtraction(
            Self::validate_integer(db, left)?,
            Self::validate_integer(db, right)?,
        ))
    }

    pub fn product(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<MathExpression> {
        Ok(MathExpression::Product(
            Self::validate_integer(db, left)?,
            Self::validate_integer(db, right)?,
        ))
    }

    pub fn division(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<MathExpression> {
        Ok(MathExpression::Division(
            Self::validate_integer(db, left)?,
            Self::validate_integer(db, right)?,
        ))
    }
}

impl DeclareWithIndent for MathExpression {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        Ok(match self {
            MathExpression::Negative(val) => {
                format!("-{}", val.declare_with_indent(db, indent_style)?)
            }
            MathExpression::Sum(left, right) => format!(
                "{} + {}",
                left.declare_with_indent(db, indent_style)?,
                right.declare_with_indent(db, indent_style)?
            ),
            MathExpression::Subtraction(left, right) => format!(
                "{} - {}",
                left.declare_with_indent(db, indent_style)?,
                right.declare_with_indent(db, indent_style)?
            ),
            MathExpression::Product(left, right) => format!(
                "{} * {}",
                left.declare_with_indent(db, indent_style)?,
                right.declare_with_indent(db, indent_style)?
            ),
            MathExpression::Division(left, right) => format!(
                "{} / {}",
                left.declare_with_indent(db, indent_style)?,
                right.declare_with_indent(db, indent_style)?
            ),
        })
    }
}

pub trait CreateMath: Sized {
    fn validate_integer(
        db: &dyn Arch,
        relation: impl TryResult<Relation>,
    ) -> Result<Box<Relation>> {
        let relation = relation.try_result()?;
        relation.is_integer(db)?;
        Ok(Box::new(relation))
    }
    fn r_negative(self, db: &dyn Arch) -> Result<MathExpression>;
    fn r_add(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn r_subtract(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn r_multiply(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn r_divide_by(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
}

impl<T: TryResult<Relation>> CreateMath for T {
    fn r_negative(self, db: &dyn Arch) -> Result<MathExpression> {
        MathExpression::negative(db, self)
    }

    fn r_add(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        MathExpression::sum(db, self, right)
    }

    fn r_subtract(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        MathExpression::subtraction(db, self, right)
    }

    fn r_multiply(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        MathExpression::product(db, self, right)
    }

    fn r_divide_by(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        MathExpression::division(db, self, right)
    }
}
