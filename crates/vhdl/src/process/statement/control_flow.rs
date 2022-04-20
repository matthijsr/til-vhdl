use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch,
    common::vhdl_name::VhdlName,
    declaration::DeclareWithIndent,
    statement::label::Label,
    usings::{ListUsingsDb, Usings},
};

use super::{
    case::Case,
    ifelse::IfElse,
    loop_statement::{Exit, LoopStatement},
    wait::Wait,
    SequentialStatement,
};

pub type Block = Vec<SequentialStatement>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlFlowKind {
    IfElse(IfElse),
    Case(Case),
    Loop(LoopStatement),
    Wait(Wait),
    Exit(Exit),
}

impl From<IfElse> for ControlFlowKind {
    fn from(ie: IfElse) -> Self {
        Self::IfElse(ie)
    }
}

impl From<Case> for ControlFlowKind {
    fn from(ie: Case) -> Self {
        Self::Case(ie)
    }
}

impl From<LoopStatement> for ControlFlowKind {
    fn from(ie: LoopStatement) -> Self {
        Self::Loop(ie)
    }
}

impl From<Wait> for ControlFlowKind {
    fn from(ie: Wait) -> Self {
        Self::Wait(ie)
    }
}

impl From<Exit> for ControlFlowKind {
    fn from(ie: Exit) -> Self {
        Self::Exit(ie)
    }
}

impl DeclareWithIndent for ControlFlowKind {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        match self {
            ControlFlowKind::IfElse(_) => todo!(),
            ControlFlowKind::Case(_) => todo!(),
            ControlFlowKind::Loop(_) => todo!(),
            ControlFlowKind::Wait(w) => w.declare_with_indent(db, indent_style),
            ControlFlowKind::Exit(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlFlow {
    label: Option<VhdlName>,
    kind: ControlFlowKind,
}

impl ControlFlow {
    /// Get a reference to the control flow's kind.
    #[must_use]
    pub fn kind(&self) -> &ControlFlowKind {
        &self.kind
    }
}

impl<T: Into<ControlFlowKind>> From<T> for ControlFlow {
    fn from(val: T) -> Self {
        Self {
            label: None,
            kind: val.into(),
        }
    }
}

impl DeclareWithIndent for ControlFlow {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        self.kind().declare_with_indent(db, indent_style)
    }
}

impl Label for ControlFlow {
    fn label(&self) -> Option<&VhdlName> {
        self.label.as_ref()
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        self.label = Some(label.into())
    }
}

impl ListUsingsDb for ControlFlow {
    fn list_usings_db(&self, _db: &dyn Arch) -> Result<Usings> {
        match self.kind() {
            ControlFlowKind::IfElse(_) => todo!(),
            ControlFlowKind::Case(_) => todo!(),
            ControlFlowKind::Loop(_) => todo!(),
            ControlFlowKind::Wait(_) => Ok(Usings::new_empty()),
            ControlFlowKind::Exit(_) => todo!(),
        }
    }
}
