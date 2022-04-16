use tydi_intern::Id;

use crate::{assignment::ValueAssignment, declaration::ObjectDeclaration};

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
