use super::{
    condition::{DefaultConditions, GenericCondition},
    VerifyConditions,
};
use tydi_common::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceGenericKind {
    Dimensionality,
}

impl VerifyConditions for InterfaceGenericKind {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        match self {
            InterfaceGenericKind::Dimensionality => conditions
                .iter()
                .try_for_each(|c| c.verify_min_max("Dimensionality", 2, i32::MAX)),
        }
    }
}

impl DefaultConditions for InterfaceGenericKind {
    fn default_conditions(&self) -> Vec<GenericCondition> {
        match self {
            InterfaceGenericKind::Dimensionality => vec![
                GenericCondition::gteq("2"),
                GenericCondition::lteq(i32::MAX.to_string()),
            ],
        }
    }
}
