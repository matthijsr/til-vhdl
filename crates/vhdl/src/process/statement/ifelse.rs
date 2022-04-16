use super::{condition::Condition, Block};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConditionalBlock {
    condition: Condition,
    block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfElse {
    if_block: ConditionalBlock,
    else_ifs: Vec<ConditionalBlock>,
    else_block: Option<ConditionalBlock>,
}
