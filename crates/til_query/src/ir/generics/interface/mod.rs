use super::VerifyConditions;
use tydi_common::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceGenericKind {
    Dimensionality,
}

impl VerifyConditions for InterfaceGenericKind {
    fn verify_conditions(&self, conditions: &[super::condition::GenericCondition]) -> Result<()> {
        match self {
            InterfaceGenericKind::Dimensionality => conditions
                .iter()
                .try_for_each(|c| c.verify_min_max("Dimensionality", 2, i32::MAX)),
        }
    }
}
