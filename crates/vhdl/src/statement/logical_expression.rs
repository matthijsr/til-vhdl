use tydi_common::error::{Error, Result, TryResult};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::{interner::GetName, Arch},
    assignment::ValueAssignment,
    declaration::{DeclareWithIndent, ObjectDeclaration},
    object::object_type::ObjectType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalExpression {
    left: Box<Relation>,
    right: Box<Relation>,
    operator: LogicalOperator,
}

impl LogicalExpression {
    /// Get a reference to the logical expression's left.
    #[must_use]
    pub fn left(&self) -> &Relation {
        self.left.as_ref()
    }

    /// Get a reference to the logical expression's right.
    #[must_use]
    pub fn right(&self) -> &Relation {
        self.right.as_ref()
    }

    /// Get a reference to the logical expression's operator.
    #[must_use]
    pub fn operator(&self) -> &LogicalOperator {
        &self.operator
    }

    pub fn can_assign(&self, db: &dyn Arch, to_typ: &ObjectType) -> Result<()> {
        self.left().can_assign(db, to_typ)
    }
}

pub trait CreateLogicalExpression: Sized {
    fn validate_lex(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<(Box<Relation>, Box<Relation>)> {
        let left = left.try_result()?;
        let right = right.try_result()?;
        left.matching_relation(db, &right)?;
        Ok((Box::new(left), Box::new(right)))
    }

    fn and(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn or(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn xor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn nand(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn nor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
    fn xnor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression>;
}

impl<T: TryResult<Relation>> CreateLogicalExpression for T {
    fn and(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::And,
        })
    }

    fn or(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Or,
        })
    }

    fn xor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Xor,
        })
    }

    fn nand(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Nand,
        })
    }

    fn nor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Nor,
        })
    }

    fn xnor(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<LogicalExpression> {
        let (left, right) = Self::validate_lex(db, self, right)?;
        Ok(LogicalExpression {
            left,
            right,
            operator: LogicalOperator::Xnor,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationalOperator {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelationalCombination {
    left: Box<Relation>,
    right: Box<Relation>,
    operator: RelationalOperator,
}

pub trait CombineRelation: Sized {
    fn validate_combine(
        db: &dyn Arch,
        left: impl TryResult<Relation>,
        right: impl TryResult<Relation>,
    ) -> Result<(Box<Relation>, Box<Relation>)> {
        let left = left.try_result()?;
        let right = right.try_result()?;
        left.matching_relation(db, &right)?;
        Ok((Box::new(left), Box::new(right)))
    }

    fn eq(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn not_eq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination>;
    fn lt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn lt_eq(self, db: &dyn Arch, right: impl TryResult<Relation>)
        -> Result<RelationalCombination>;
    fn gt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination>;
    fn gt_eq(self, db: &dyn Arch, right: impl TryResult<Relation>)
        -> Result<RelationalCombination>;
}

impl<T: TryResult<Relation>> CombineRelation for T {
    fn eq(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Eq,
        })
    }

    fn not_eq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::NotEq,
        })
    }

    fn lt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Lt,
        })
    }

    fn lt_eq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::LtEq,
        })
    }

    fn gt(self, db: &dyn Arch, right: impl TryResult<Relation>) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::Gt,
        })
    }

    fn gt_eq(
        self,
        db: &dyn Arch,
        right: impl TryResult<Relation>,
    ) -> Result<RelationalCombination> {
        let (left, right) = Self::validate_combine(db, self, right)?;
        Ok(RelationalCombination {
            left,
            right,
            operator: RelationalOperator::GtEq,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    Value(Box<ValueAssignment>),
    Object(Id<ObjectDeclaration>),
    Combination(RelationalCombination),
    LogicalExpression(LogicalExpression),
}

impl Relation {
    pub fn can_assign(&self, db: &dyn Arch, to_typ: &ObjectType) -> Result<()> {
        match self {
            Relation::Value(v) => v.can_assign(to_typ),
            Relation::Object(o) => db.get_object_declaration_type(*o)?.can_assign_type(to_typ),
            Relation::Combination(_) => ObjectType::Boolean.can_assign_type(to_typ),
            Relation::LogicalExpression(_) => todo!(),
        }
    }

    pub fn matching_relation(&self, db: &dyn Arch, other: &Relation) -> Result<()> {
        match self {
            Relation::Value(v) => match other {
                Relation::Value(ov) => {
                    if v.matching_value(ov) {
                        Ok(())
                    } else {
                        Err(Error::InvalidArgument(format!(
                            "Cannot create a relation between {} and {}.",
                            v.declare()?,
                            ov.declare()?
                        )))
                    }
                }
                Relation::Object(o) => v.can_assign(db.get_object_declaration_type(*o)?.as_ref()),
                Relation::Combination(_) => match v.as_ref() {
                    ValueAssignment::Boolean(_) => Ok(()),
                    _ => Err(Error::InvalidArgument(format!(
                        "Cannot create a relation between {} and a boolean relation.",
                        v.declare()?,
                    ))),
                },
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::Object(o) => match other {
                Relation::Value(v) => v.can_assign(db.get_object_declaration_type(*o)?.as_ref()),
                Relation::Object(oo) => db
                    .get_object_declaration_type(*o)?
                    .can_assign_type(db.get_object_declaration_type(*oo)?.as_ref()),
                Relation::Combination(_) => ValueAssignment::Boolean(false)
                    .can_assign(db.get_object_declaration_type(*o)?.as_ref()),
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::Combination(_) => match other {
                Relation::Value(v) => match v.as_ref() {
                    ValueAssignment::Boolean(_) => Ok(()),
                    _ => Err(Error::InvalidArgument(format!(
                        "Cannot create a relation between {} and a boolean relation.",
                        v.declare()?,
                    ))),
                },
                Relation::Object(o) => ValueAssignment::Boolean(false)
                    .can_assign(db.get_object_declaration_type(*o)?.as_ref()),
                Relation::Combination(_) => Ok(()),
                Relation::LogicalExpression(lex) => self.matching_relation(db, lex.left()),
            },
            Relation::LogicalExpression(lex) => lex.right().matching_relation(db, other),
        }
    }
}

impl DeclareWithIndent for Relation {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            Relation::Value(v) => v.declare(),
            Relation::Object(obj) => Ok(obj.get_name(db).to_string()),
            Relation::Combination(_) => todo!(),
            Relation::LogicalExpression(_) => todo!(),
        }
    }
}
