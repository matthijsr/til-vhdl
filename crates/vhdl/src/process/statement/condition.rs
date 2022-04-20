use tydi_common::error::{Result, TryResult};

use crate::{
    architecture::arch_storage::Arch, declaration::DeclareWithIndent, statement::relation::Relation,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Condition {
    Constant(bool),
    Relation(Relation),
}

impl Condition {
    pub fn constant(val: bool) -> Self {
        Self::Constant(val)
    }

    pub fn relation(db: &dyn Arch, relation: impl TryResult<Relation>) -> Result<Self> {
        let relation = relation.try_result()?;
        relation.is_bool(db)?;

        Ok(Self::Relation(relation))
    }
}

impl From<bool> for Condition {
    fn from(val: bool) -> Self {
        Condition::constant(val)
    }
}

impl DeclareWithIndent for Condition {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            Condition::Constant(val) => Ok(val.to_string()),
            Condition::Relation(rel) => rel.declare_with_indent(db, indent_style),
        }
    }
}
