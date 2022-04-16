use crate::statement::logical_expression::Relation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BooleanValue {
    Constant(bool),
    Relation(Relation),
}
