use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch, common::vhdl_name::VhdlName, process::Process,
    usings::ListUsingsDb,
};

use self::{label::Label, mapping::Mapping};

use super::assignment::AssignDeclaration;

pub mod declare;
pub mod label;
pub mod mapping;
pub mod relation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Assignment(AssignDeclaration),
    PortMapping(Mapping),
    Process(Process),
}

impl ListUsingsDb for Statement {
    fn list_usings_db(&self, db: &dyn Arch) -> Result<crate::usings::Usings> {
        match self {
            Statement::Assignment(a) => a.list_usings_db(db),
            Statement::PortMapping(pm) => pm.list_usings_db(db),
            Statement::Process(p) => p.list_usings_db(db),
        }
    }
}

impl Label for Statement {
    fn label(&self) -> Option<&VhdlName> {
        match self {
            Statement::Assignment(a) => a.label(),
            Statement::PortMapping(p) => p.label(),
            Statement::Process(p) => p.label(),
        }
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        match self {
            Statement::Assignment(a) => a.set_label(label),
            Statement::PortMapping(p) => p.set_label(label),
            Statement::Process(p) => p.set_label(label),
        }
    }
}

impl From<AssignDeclaration> for Statement {
    fn from(assign: AssignDeclaration) -> Self {
        Statement::Assignment(assign)
    }
}

impl From<Mapping> for Statement {
    fn from(portmapping: Mapping) -> Self {
        Statement::PortMapping(portmapping)
    }
}

impl From<Process> for Statement {
    fn from(process: Process) -> Self {
        Statement::Process(process)
    }
}
