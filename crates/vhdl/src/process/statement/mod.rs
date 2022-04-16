pub mod case;
pub mod condition;
pub mod ifelse;
pub mod loop_statement;
pub mod test_statement;
pub mod wait;

use crate::{assignment::AssignDeclaration, common::vhdl_name::VhdlName};

use self::{
    case::Case, ifelse::IfElse, loop_statement::{LoopStatement, Exit}, test_statement::TestStatement,
    wait::Wait,
};

pub type Block = Vec<SequentialStatement>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlFlow {
    IfElse(IfElse),
    Case(Case),
    Loop(LoopStatement),
    Wait(Wait),
    Exit(Exit),
}

// REFER TO: https://insights.sigasi.com/tech/vhdl2008.ebnf/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequentialStatementKind {
    Assignment(AssignDeclaration),
    Control(ControlFlow),
    Test(TestStatement),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequentialStatement {
    label: Option<VhdlName>,
    kind: SequentialStatementKind,
}
