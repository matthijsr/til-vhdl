use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf},
    traits::Identify,
};

use self::{
    behavioral::BehavioralGenericKind,
    condition::TestValue,
    interface::{dimensionality::DimensionalityGeneric, InterfaceGenericKind},
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

impl<B: Into<BehavioralGenericKind>> From<B> for GenericKind {
    fn from(val: B) -> Self {
        Self::Behavioral(val.into())
    }
}

impl From<InterfaceGenericKind> for GenericKind {
    fn from(val: InterfaceGenericKind) -> Self {
        Self::Interface(val)
    }
}

impl From<DimensionalityGeneric> for GenericKind {
    fn from(val: DimensionalityGeneric) -> Self {
        Self::Interface(val.into())
    }
}

impl TestValue for GenericKind {
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        match self {
            GenericKind::Behavioral(behav) => behav.valid_value(value),
            GenericKind::Interface(iface) => iface.valid_value(value),
        }
    }

    fn describe_condition(&self) -> String {
        match self {
            GenericKind::Behavioral(behav) => behav.describe_condition(),
            GenericKind::Interface(iface) => iface.describe_condition(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericParameter {
    name: Name,
    kind: GenericKind,
    default_value: GenericParamValue,
}

impl GenericParameter {
    pub fn try_new(
        name: impl TryResult<Name>,
        kind: impl TryResult<GenericKind>,
        default_value: impl TryResult<GenericParamValue>,
    ) -> Result<Self> {
        let r = Self {
            name: name.try_result()?,
            kind: kind.try_result()?,
            default_value: default_value.try_result()?,
        };
        if r.valid_value(r.default_value().clone())? {
            Ok(r)
        } else {
            Err(Error::InvalidArgument(format!(
                "Default value ({}) is not valid for condition: {}",
                r.default_value(),
                r.describe_condition()
            )))
        }
    }

    pub fn kind(&self) -> &GenericKind {
        &self.kind
    }

    pub fn default_value(&self) -> &GenericParamValue {
        &self.default_value
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
    fn valid_value(&self, value: impl TryResult<GenericParamValue>) -> Result<bool> {
        self.kind().valid_value(value)
    }

    fn describe_condition(&self) -> String {
        self.kind().describe_condition()
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::generics::behavioral::integer::IntegerGeneric;

    use super::{
        condition::{integer_condition::IntegerCondition, AppliesCondition, BuildsCondition},
        *,
    };

    #[test]
    fn test_conditions() -> Result<()> {
        let param = GenericParameter::try_new(
            "a",
            IntegerGeneric::natural()
                .with_condition(IntegerCondition::Eq(2).or(IntegerCondition::Gt(5)).invert())?,
            0,
        )?;
        assert_eq!(
            "(Natural, implicit: >= 0) and !(== 2 or > 5)",
            param.describe_condition()
        );
        assert_eq!(param.valid_value(-1)?, false);
        assert_eq!(param.valid_value(0)?, true);
        assert_eq!(param.valid_value(1)?, true);
        assert_eq!(param.valid_value(2)?, false);
        assert_eq!(param.valid_value(3)?, true);
        assert_eq!(param.valid_value(4)?, true);
        assert_eq!(param.valid_value(5)?, true);
        assert_eq!(param.valid_value(6)?, false);

        Ok(())
    }
}
