#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericCondition {
    Gt(String),
    Lt(String),
    GtEq(String),
    LtEq(String),
}
