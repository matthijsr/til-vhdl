use crate::ir::generics::VerifyConditions;
use tydi_common::error::Result;

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
    ) -> Result<()> {
        todo!()
    }
}
