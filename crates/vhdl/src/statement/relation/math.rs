use tydi_common::error::{Result, TryResult};

use crate::{
    architecture::arch_storage::Arch,
    declaration::DeclareWithIndent,
    object::object_type::{IntegerType, ObjectType},
};

use super::Relation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathExpression {
    Negative(Box<Relation>),
    Sum(Box<Relation>, Box<Relation>),
    Subtraction(Box<Relation>, Box<Relation>),
    Product(Box<Relation>, Box<Relation>),
    Division(Box<Relation>, Box<Relation>),
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
    fn validate_integer(db: &dyn Arch, relation: impl TryResult<Relation>) -> Result<Relation> {
        let relation = relation.try_result()?;
        relation.can_assign(db, &ObjectType::Integer(IntegerType::Integer))?;
        Ok(relation)
    }
    fn negative(self, db: &dyn Arch) -> Result<MathExpression>;
    fn add(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn subtract(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn multiply(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
    fn divide_by(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression>;
}

impl<T: TryResult<Relation>> CreateMath for T {
    fn negative(self, db: &dyn Arch) -> Result<MathExpression> {
        todo!()
    }

    fn add(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        todo!()
    }

    fn subtract(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        todo!()
    }

    fn multiply(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        todo!()
    }

    fn divide_by(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<MathExpression> {
        todo!()
    }
}
