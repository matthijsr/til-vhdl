use self::number::NumberGenericKind;
use tydi_common::error::Result;

use super::{
    condition::{DefaultConditions, GenericCondition},
    VerifyConditions,
};

pub mod number;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehavioralGenericKind {
    Number(NumberGenericKind),
}

impl VerifyConditions for BehavioralGenericKind {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        match self {
            BehavioralGenericKind::Number(n) => n.verify_conditions(conditions),
        }
    }
}

impl DefaultConditions for BehavioralGenericKind {
    fn default_conditions(&self) -> Vec<GenericCondition> {
        match self {
            BehavioralGenericKind::Number(n) => n.default_conditions(),
        }
    }
}
