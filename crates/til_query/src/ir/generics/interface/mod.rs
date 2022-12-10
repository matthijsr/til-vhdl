use super::VerifyConditions;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceGenericKind {
    Dimensionality,
}

impl VerifyConditions for InterfaceGenericKind {
    fn verify_conditions(
        &self,
        conditions: &[super::condition::GenericCondition],
    ) -> tydi_common::error::Result<()> {
        todo!()
    }
}
