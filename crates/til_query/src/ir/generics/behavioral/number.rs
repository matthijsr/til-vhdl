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
