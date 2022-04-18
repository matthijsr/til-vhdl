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

impl Wait {
    /// `... on [sensitivity list]`
    pub fn sensitivity(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.sensitivity
    }
    /// `... for [condition]`
    pub fn condition(&self) -> &Option<Condition> {
        &self.condition
    }
    /// `... until [timeout]`
    pub fn timeout(&self) -> &Option<TimeExpression> {
        &self.timeout
    }
}
