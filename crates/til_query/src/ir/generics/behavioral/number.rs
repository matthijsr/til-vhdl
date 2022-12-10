use crate::ir::generics::{
    condition::{DefaultConditions, GenericCondition},
    VerifyConditions,
};
use tydi_common::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberGenericKind {
    Integer,
    Natural,
    Positive,
}

impl VerifyConditions for NumberGenericKind {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        match self {
            NumberGenericKind::Integer => conditions
                .iter()
                .try_for_each(|c| c.verify_min_max("Integer", i32::MIN, i32::MAX)),
            NumberGenericKind::Natural => conditions
                .iter()
                .try_for_each(|c| c.verify_min_max("Natural", 0, i32::MAX)),
            NumberGenericKind::Positive => conditions
                .iter()
                .try_for_each(|c| c.verify_min_max("Positive", 1, i32::MAX)),
        }
    }
}

impl DefaultConditions for NumberGenericKind {
    fn default_conditions(&self) -> Vec<GenericCondition> {
        match self {
            NumberGenericKind::Integer => vec![
                GenericCondition::gteq(i32::MIN.to_string()),
                GenericCondition::lteq(i32::MAX.to_string()),
            ],
            NumberGenericKind::Natural => vec![
                GenericCondition::gteq("0"),
                GenericCondition::lteq(i32::MAX.to_string()),
            ],
            NumberGenericKind::Positive => vec![
                GenericCondition::gteq("1"),
                GenericCondition::lteq(i32::MAX.to_string()),
            ],
        }
    }
}
