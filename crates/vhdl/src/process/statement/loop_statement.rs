use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoopStatement {
    While(Condition),
    For,
}
