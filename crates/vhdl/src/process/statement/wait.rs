use super::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Wait {
    Wait,
    WaitUntil(Condition),
}
