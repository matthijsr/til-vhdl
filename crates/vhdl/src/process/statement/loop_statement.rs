use crate::common::vhdl_name::VhdlName;

use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Exit {
    loop_label: Option<VhdlName>,
    condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoopStatement {
    While(Condition),
    For,
}
