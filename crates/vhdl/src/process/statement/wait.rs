use tydi_common::map::InsertionOrderedMap;
use tydi_intern::Id;

use crate::{
    common::vhdl_name::VhdlName, declaration::ObjectDeclaration,
    object::object_type::time::TimeValue,
};

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeExpression {
    Constant(TimeValue),
    Variable(Id<ObjectDeclaration>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A `wait` statement, with optional clauses.
/// 
/// Hence, this will declare as `wait`, by default.
pub struct Wait {
    /// `... on [sensitivity list]`
    sensitivity: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// `... for [condition]`
    condition: Option<Condition>,
    /// `... until [timeout]`
    timeout: Option<TimeExpression>,
}
