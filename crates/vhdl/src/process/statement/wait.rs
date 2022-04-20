use itertools::Itertools;
use tydi_common::{
    error::{Error, Result, TryResult},
    map::InsertionOrderedMap,
};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    assignment::ObjectSelection,
    common::vhdl_name::VhdlName,
    declaration::{DeclareWithIndent, ObjectDeclaration},
    object::object_type::{time::TimeValue, ObjectType},
    statement::relation::Relation,
};

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeExpression {
    Constant(TimeValue),
    Variable(ObjectSelection),
}

impl TimeExpression {
    pub fn variable(db: &dyn Arch, obj: impl TryResult<ObjectSelection>) -> Result<Self> {
        let obj = obj.try_result()?;
        let typ = db.get_object_type(obj.as_object_key(db))?;
        match typ.as_ref() {
            ObjectType::Time => Ok(Self::Variable(obj)),
            ObjectType::Bit
            | ObjectType::Array(_)
            | ObjectType::Record(_)
            | ObjectType::Boolean => Err(Error::InvalidArgument(format!(
                "Object with type {} cannot be used for a Time expression.",
                typ
            ))),
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
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            TimeExpression::Constant(t) => t.declare(),
            TimeExpression::Variable(v) => v.declare_with_indent(db, indent_style),
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
    /// `... until [condition]`
    condition: Option<Condition>,
    /// `... for [timeout]`
    timeout: Option<TimeExpression>,
}

impl Wait {
    /// `... on [sensitivity list]`
    pub fn sensitivity(&self) -> &InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>> {
        &self.sensitivity
    }
    /// `... until [condition]`
    pub fn condition(&self) -> Option<&Condition> {
        self.condition.as_ref()
    }
    /// `... for [timeout]`
    pub fn timeout(&self) -> Option<&TimeExpression> {
        self.timeout.as_ref()
    }

    pub fn wait() -> Self {
        Self {
            sensitivity: InsertionOrderedMap::new(),
            condition: None,
            timeout: None,
        }
    }

    pub fn on(mut self, db: &dyn Arch, obj: Id<ObjectDeclaration>) -> Result<Self> {
        self.sensitivity
            .try_insert(db.get_object_declaration_name(obj).as_ref().clone(), obj)?;
        Ok(self)
    }

    pub fn until_constant(mut self, val: bool) -> Self {
        self.condition = Some(Condition::constant(val));
        self
    }

    pub fn until_relation(
        mut self,
        db: &dyn Arch,
        relation: impl TryResult<Relation>,
    ) -> Result<Self> {
        self.condition = Some(Condition::relation(db, relation)?);
        Ok(self)
    }

    pub fn for_constant(mut self, val: impl Into<TimeValue>) -> Self {
        self.timeout = Some(TimeExpression::constant(val));
        self
    }

    pub fn for_variable(
        mut self,
        db: &dyn Arch,
        obj: impl TryResult<ObjectSelection>,
    ) -> Result<Self> {
        self.timeout = Some(TimeExpression::variable(db, obj)?);
        Ok(self)
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
            result.push_str(" until ");
            result.push_str(&condition.declare_with_indent(db, indent_style)?);
        }

        if let Some(timeout) = self.timeout() {
            result.push_str(" for ");
            result.push_str(&timeout.declare_with_indent(db, indent_style)?);
        }

        Ok(result)
    }
}
