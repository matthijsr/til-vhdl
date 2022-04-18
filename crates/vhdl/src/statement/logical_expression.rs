use tydi_common::error::Result;
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::{interner::GetName, Arch},
    assignment::ValueAssignment,
    declaration::{DeclareWithIndent, ObjectDeclaration},
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    Value(ValueAssignment),
    Object(Id<ObjectDeclaration>),
    Combination(RelationalCombination),
    LogicalExpression(LogicalExpression),
}

impl DeclareWithIndent for Relation {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            Relation::Value(_) => todo!(),
            Relation::Object(obj) => Ok(obj.get_name(db).to_string()),
            Relation::Combination(_) => todo!(),
            Relation::LogicalExpression(_) => todo!(),
        }
    }
}
