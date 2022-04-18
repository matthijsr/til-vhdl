use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch, declaration::DeclareWithIndent,
    statement::logical_expression::Relation,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BooleanValue {
    Constant(bool),
    Relation(Relation),
}

impl DeclareWithIndent for BooleanValue {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            BooleanValue::Constant(val) => Ok(val.to_string()),
            BooleanValue::Relation(rel) => rel.declare_with_indent(db, indent_style),
        }
    }
}
