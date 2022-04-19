use itertools::Itertools;
use tydi_common::{
    error::{Error, Result},
    map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::{interner::GetName, Arch},
    common::vhdl_name::VhdlName,
    declaration::{DeclareWithIndent, ObjectDeclaration},
    object::object_type::{time::TimeValue, ObjectType},
};

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeExpression {
    Constant(TimeValue),
    Variable(Id<ObjectDeclaration>),
}

impl TimeExpression {
    pub fn variable(db: &dyn Arch, obj: Id<ObjectDeclaration>) -> Result<Self> {
        let typ = db.get_object_declaration_type(obj)?;
        match typ.as_ref() {
            ObjectType::Time => Ok(Self::Variable(obj)),
            ObjectType::Bit | ObjectType::Array(_) | ObjectType::Record(_) => {
                Err(Error::InvalidArgument(format!(
                    "Object with type {} cannot be used for a Time expression.",
                    typ
                )))
            }
        }
    }

    pub fn constant(val: impl Into<TimeValue>) -> Self {
        TimeExpression::Constant(val.into())
    }
}

impl From<TimeValue> for TimeExpression {
    fn from(val: TimeValue) -> Self {
        TimeExpression::Constant(val)
    }
}

impl DeclareWithIndent for TimeExpression {
    fn declare_with_indent(&self, db: &dyn Arch, _indent_style: &str) -> Result<String> {
        match self {
            TimeExpression::Constant(t) => t.declare(),
            TimeExpression::Variable(v) => Ok(v.get_name(db).to_string()),
        }
    }
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
    pub fn condition(&self) -> Option<&Condition> {
        self.condition.as_ref()
    }
    /// `... until [timeout]`
    pub fn timeout(&self) -> Option<&TimeExpression> {
        self.timeout.as_ref()
    }
}

impl DeclareWithIndent for Wait {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let mut result = "wait".to_string();
        if self.sensitivity().len() > 0 {
            result.push_str(" on ");
            result.push_str(&self.sensitivity().keys().join(", "))
        }

        if let Some(condition) = self.condition() {
            result.push_str(" for ");
            result.push_str(&condition.declare_with_indent(db, indent_style)?);
        }

        if let Some(timeout) = self.timeout() {
            result.push_str(" until ");
            result.push_str(&timeout.declare_with_indent(db, indent_style)?);
        }

        Ok(result)
    }
}