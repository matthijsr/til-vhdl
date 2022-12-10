use std::{fmt, str::FromStr};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf},
    traits::Identify,
};

use self::{
    behavioral::{number::NumberGenericKind, BehavioralGenericKind},
    condition::{DefaultConditions, GenericCondition},
    interface::InterfaceGenericKind,
};

pub mod behavioral;
pub mod condition;
pub mod interface;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericKind {
    Behavioral(BehavioralGenericKind),
    Interface(InterfaceGenericKind),
}

impl TryFrom<&str> for GenericKind {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::from_str(value)
    }
}

impl FromStr for GenericKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let val = s.to_lowercase();
        match val.as_str() {
            "integer" => Ok(Self::Behavioral(BehavioralGenericKind::Number(
                NumberGenericKind::Integer,
            ))),
            "natural" => Ok(Self::Behavioral(BehavioralGenericKind::Number(
                NumberGenericKind::Natural,
            ))),
            "positive" => Ok(Self::Behavioral(BehavioralGenericKind::Number(
                NumberGenericKind::Positive,
            ))),
            "dimensionality" => Ok(Self::Interface(InterfaceGenericKind::Dimensionality)),
            _ => Err(Error::InvalidArgument(format!(
                "No GenericKind matching string \"{}\"",
                val
            ))),
        }
    }
}

impl VerifyConditions for GenericKind {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        match self {
            GenericKind::Behavioral(b) => b.verify_conditions(conditions),
            GenericKind::Interface(i) => i.verify_conditions(conditions),
        }
    }
}

impl DefaultConditions for GenericKind {
    fn default_conditions(&self) -> Vec<GenericCondition> {
        match self {
            GenericKind::Behavioral(b) => b.default_conditions(),
            GenericKind::Interface(i) => i.default_conditions(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericParameter {
    name: Name,
    kind: GenericKind,
    conditions: Vec<GenericCondition>,
}

impl GenericParameter {
    pub fn try_new(name: impl TryResult<Name>, kind: impl TryResult<GenericKind>) -> Result<Self> {
        Ok(Self {
            name: name.try_result()?,
            kind: kind.try_result()?,
            conditions: vec![],
        })
    }

    pub fn set_conditions(
        &mut self,
        conditions: impl IntoIterator<Item = impl TryResult<GenericCondition>>,
    ) -> Result<()> {
        self.conditions = conditions
            .into_iter()
            .map(|x| x.try_result())
            .collect::<Result<Vec<_>>>()?;
        self.verify_conditions(self.conditions())
    }

    pub fn with_conditions(
        mut self,
        conditions: impl IntoIterator<Item = impl TryResult<GenericCondition>>,
    ) -> Result<Self> {
        self.set_conditions(conditions)?;
        Ok(self)
    }

    pub fn kind(&self) -> &GenericKind {
        &self.kind
    }

    pub fn conditions(&self) -> &[GenericCondition] {
        self.conditions.as_ref()
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

pub trait VerifyConditions {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()>;
}

impl VerifyConditions for GenericParameter {
    fn verify_conditions(&self, conditions: &[GenericCondition]) -> Result<()> {
        self.kind().verify_conditions(conditions)
    }
}

impl DefaultConditions for GenericParameter {
    fn default_conditions(&self) -> Vec<GenericCondition> {
        self.kind().default_conditions()
    }
}

pub trait TestValue {
    fn test_value<T: FromStr<Err = impl fmt::Display> + PartialOrd + fmt::Display>(
        &self,
        value: &T,
    ) -> Result<()>;
}

impl TestValue for GenericParameter {
    fn test_value<T: FromStr<Err = impl fmt::Display> + PartialOrd + fmt::Display>(
        &self,
        value: &T,
    ) -> Result<()> {
        let defaults = self.default_conditions();
        for default_condition in defaults {
            default_condition.test_value(value)?;
        }
        for condition in self.conditions() {
            condition.test_value(value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conditions() -> Result<()> {
        let param_attempt = GenericParameter::try_new("a", "positive")?
            .with_conditions(["<= 1", ">1", " >= 1", "< 1"]);
        assert_eq!(param_attempt.is_err(), true);
        let param_attempt = GenericParameter::try_new("a", "positive")?
            .with_conditions(["<= 1", ">1", " >= 1", "< 2"]);
        assert_eq!(param_attempt.is_ok(), true);

        let mut param = GenericParameter::try_new("a", "positive")?;
        assert_eq!(param.test_value(&0).is_ok(), false);
        assert_eq!(param.test_value(&1).is_ok(), true);

        assert_eq!(param.test_value(&2).is_ok(), true);
        param.set_conditions(["> 2"])?;
        assert_eq!(param.test_value(&1).is_ok(), false);
        assert_eq!(param.test_value(&2).is_ok(), false);
        assert_eq!(param.test_value(&3).is_ok(), true);

        assert_eq!(param.test_value(&4).is_ok(), true);
        param.set_conditions(["> 2", "< 4"])?;
        assert_eq!(param.test_value(&1).is_ok(), false);
        assert_eq!(param.test_value(&2).is_ok(), false);
        assert_eq!(param.test_value(&3).is_ok(), true);
        assert_eq!(param.test_value(&4).is_ok(), false);

        param.set_conditions([">= 2", "<= 4"])?;
        assert_eq!(param.test_value(&1).is_ok(), false);
        assert_eq!(param.test_value(&2).is_ok(), true);
        assert_eq!(param.test_value(&3).is_ok(), true);
        assert_eq!(param.test_value(&4).is_ok(), true);
        assert_eq!(param.test_value(&5).is_ok(), false);

        Ok(())
    }
}
