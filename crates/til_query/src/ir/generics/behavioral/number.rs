use crate::ir::generics::VerifyConditions;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberGenericKind {
    Integer,
    Natural,
    Positive,
}

impl VerifyConditions for NumberGenericKind {
    fn verify_conditions(
        &self,
        conditions: &[crate::ir::generics::condition::GenericCondition],
    ) -> tydi_common::error::Result<()> {
        todo!()
    }
}
