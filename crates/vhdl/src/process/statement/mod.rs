pub mod case;
pub mod condition;
pub mod ifelse;
pub mod loop_statement;
pub mod test_statement;
pub mod wait;

use crate::{assignment::AssignDeclaration, common::vhdl_name::VhdlName, statement::label::Label};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlFlow {
    label: Option<VhdlName>,
    kind: ControlFlowKind,
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
            SequentialStatement::Test(_) => todo!(),
        }
    }

    fn set_label(&mut self, label: impl Into<VhdlName>) {
        match self {
            SequentialStatement::Assignment(a) => a.set_label(label),
            SequentialStatement::Control(c) => c.set_label(label),
            SequentialStatement::Test(_) => todo!(),
        }
    }
}
