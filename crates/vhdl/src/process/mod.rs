pub mod statement;

use tydi_common::{error::Result, map::InsertionOrderedMap};
use tydi_intern::Id;

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    declaration::{DeclareWithIndent, ObjectDeclaration},
    statement::label::Label,
    usings::{ListUsings, Usings},
};

use self::statement::SequentialStatement;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Process {
    /// The label of this process.
    ///
    /// While not required in VHDL, this significantly improves the ability to
    /// debug issues in code generation (and in VHDL simulation itself), and is
    /// thus required by this implementation.
    label: VhdlName,
    /// The sensitivity list of this process, indexed by the names of the
    /// objects.
    sensitivity_list: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// Variable declarations on this process. Indexed by their names.
    ///
    /// While a process's declarative part can technically contain different
    /// declarations. For our purposes, only variables are relevant at this
    /// time.
    variable_declarations: InsertionOrderedMap<VhdlName, Id<ObjectDeclaration>>,
    /// The process's statements.
    statements: Vec<SequentialStatement>,
}

impl Label for Process {
    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = label.into()
    }

    fn label(&self) -> Option<&VhdlName> {
        Some(&self.label)
    }
}

impl ListUsings for Process {
    fn list_usings(&self) -> Result<Usings> {
        todo!()
    }
}

impl DeclareWithIndent for Process {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        todo!()
    }
}
