pub mod case;
pub mod condition;
pub mod ifelse;
pub mod loop_statement;
pub mod test_statement;
pub mod wait;

use tydi_common::error::Result;

use crate::{
    architecture::arch_storage::Arch,
    assignment::AssignDeclaration,
    common::vhdl_name::VhdlName,
    declaration::DeclareWithIndent,
    statement::label::Label,
    usings::{ListUsingsDb, Usings},
};

use self::{
    case::Case,
    ifelse::IfElse,
    loop_statement::{Exit, LoopStatement},
    test_statement::TestStatement,
    wait::Wait,
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

// REFER TO: https://insights.sigasi.com/tech/vhdl2008.ebnf/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequentialStatement {
    Assignment(AssignDeclaration),
    Control(ControlFlow),
    Test(TestStatement),
}

impl Label for SequentialStatement {
    fn label(&self) -> Option<&VhdlName> {
        match self {
            SequentialStatement::Assignment(a) => a.label(),
            SequentialStatement::Control(c) => c.label(),
            SequentialStatement::Test(t) => t.label(),
        }
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        match self {
            SequentialStatement::Assignment(a) => a.set_label(label),
            SequentialStatement::Control(c) => c.set_label(label),
            SequentialStatement::Test(t) => t.set_label(label),
        }
    }
}

impl ListUsingsDb for SequentialStatement {
    fn list_usings_db(&self, db: &dyn Arch) -> Result<Usings> {
        todo!()
    }
}

impl DeclareWithIndent for SequentialStatement {
    fn declare_with_indent(&self, db: &dyn Arch, indent_style: &str) -> Result<String> {
        let result = match self {
            SequentialStatement::Assignment(assignment) => {
                assignment.declare_with_indent(db, indent_style)
            }
            SequentialStatement::Control(_) => todo!(),
            SequentialStatement::Test(t) => t.declare_with_indent(db, indent_style),
        };
        if let Some(label) = self.label() {
            Ok(format!("{}: {}", label, result?))
        } else {
            result
        }
    }
}
