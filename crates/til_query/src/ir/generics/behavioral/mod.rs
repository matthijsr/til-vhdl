use self::number::NumberGenericKind;
use tydi_common::error::Result;

use super::VerifyConditions;

pub mod number;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehavioralGenericKind {
    Number(NumberGenericKind),
}

impl VerifyConditions for BehavioralGenericKind {
    fn verify_conditions(&self, conditions: &[super::condition::GenericCondition]) -> Result<()> {
        todo!()
    }
}
