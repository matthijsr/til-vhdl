use std::{fmt, str::FromStr};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf},
    traits::Identify,
};

use self::{
    behavioral::{integer::IntegerGenericKind, BehavioralGenericKind},
    condition::{GenericCondition, TestValue},
    interface::InterfaceGenericKind,
    param_value::GenericParamValue,
};

pub mod behavioral;
pub mod condition;
pub mod interface;
pub mod param_value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericKind {
    Behavioral(BehavioralGenericKind),
    Interface(InterfaceGenericKind),
}

impl From<BehavioralGenericKind> for GenericKind {
    fn from(val: BehavioralGenericKind) -> Self {
        Self::Behavioral(val)
    }
}

impl From<InterfaceGenericKind> for GenericKind {
    fn from(val: InterfaceGenericKind) -> Self {
        Self::Interface(val)
    }
}

impl TestValue for GenericKind {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            GenericKind::Behavioral(behav) => behav.test_value(value),
            GenericKind::Interface(iface) => iface.test_value(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericParameter {
    name: Name,
    kind: GenericKind,
}

impl GenericParameter {
    pub fn try_new(name: impl TryResult<Name>, kind: impl TryResult<GenericKind>) -> Result<Self> {
        Ok(Self {
            name: name.try_result()?,
            kind: kind.try_result()?,
        })
    }

    pub fn kind(&self) -> &GenericKind {
        &self.kind
    }
}

impl Identify for GenericParameter {
    fn identifier(&self) -> String {
        self.name().to_string()
    }
}

impl NameSelf for GenericParameter {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl TestValue for GenericParameter {
    fn test_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        self.kind().test_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_conditions() -> Result<()> {
    //     let param_attempt = GenericParameter::try_new("a", "positive")?
    //         .with_conditions(["<= 1", ">1", " >= 1", "< 1"]);
    //     assert_eq!(param_attempt.is_err(), true);
    //     let param_attempt = GenericParameter::try_new("a", "positive")?
    //         .with_conditions(["<= 1", ">1", " >= 1", "< 2"]);
    //     assert_eq!(param_attempt.is_ok(), true);

    //     let mut param = GenericParameter::try_new("a", "positive")?;
    //     assert_eq!(param.test_value(&0).is_ok(), false);
    //     assert_eq!(param.test_value(&1).is_ok(), true);

    //     assert_eq!(param.test_value(&2).is_ok(), true);
    //     param.set_conditions(["> 2"])?;
    //     assert_eq!(param.test_value(&1).is_ok(), false);
    //     assert_eq!(param.test_value(&2).is_ok(), false);
    //     assert_eq!(param.test_value(&3).is_ok(), true);

    //     assert_eq!(param.test_value(&4).is_ok(), true);
    //     param.set_conditions(["> 2", "< 4"])?;
    //     assert_eq!(param.test_value(&1).is_ok(), false);
    //     assert_eq!(param.test_value(&2).is_ok(), false);
    //     assert_eq!(param.test_value(&3).is_ok(), true);
    //     assert_eq!(param.test_value(&4).is_ok(), false);

    //     param.set_conditions([">= 2", "<= 4"])?;
    //     assert_eq!(param.test_value(&1).is_ok(), false);
    //     assert_eq!(param.test_value(&2).is_ok(), true);
    //     assert_eq!(param.test_value(&3).is_ok(), true);
    //     assert_eq!(param.test_value(&4).is_ok(), true);
    //     assert_eq!(param.test_value(&5).is_ok(), false);

    //     Ok(())
    // }
}
