pub mod case;
pub mod condition;
pub mod ifelse;
pub mod loop_statement;
pub mod test_statement;
pub mod wait;

use crate::{assignment::AssignDeclaration, common::vhdl_name::VhdlName};

use self::{
    case::Case, ifelse::IfElse, loop_statement::LoopStatement, test_statement::TestStatement,
    wait::Wait,
};

pub type Block = Vec<SequentialStatement>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlFlow {
    IfElse(IfElse),
    Case(Case),
    Loop(LoopStatement),
    Wait(Wait),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequentialStatementKind {
    Assignment(AssignDeclaration),
    Control(ControlFlow),
    Test(TestStatement),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequentialStatement {
    label: VhdlName,
    kind: SequentialStatementKind,
}
