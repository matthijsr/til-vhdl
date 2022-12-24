use core::fmt;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::{Name, NameSelf},
    traits::{Document, Documents, Identify},
};

use self::{
    behavioral::BehavioralGenericKind,
    condition::{AppliesCondition, TestValue},
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

impl GenericKind {
    /// Verify whether this GenericKind satisfies the requirements of another
    /// GenericKind (i.e., they should be the same type, and the should have
    /// a condition that does not allow for values exceeding the other)
    pub fn satisfies(&self, other: &Self) -> Result<()> {
        match self {
            GenericKind::Behavioral(b) => {
                match b {
                    BehavioralGenericKind::Integer(i) => {
                        if let GenericKind::Behavioral(BehavioralGenericKind::Integer(other_i)) =
                            other
                        {
                            if i.kind() == other_i.kind() {
                                if i.condition().satisfies(other_i.condition()) {
                                    Ok(())
                                } else {
                                    Err(Error::InvalidArgument(format!(
                                        "Condition \"{}\" is more permissive than condition \"{}\"",
                                        i.describe_condition(),
                                        other_i.describe_condition()
                                    )))
                                }
                            } else {
                                Err(Error::InvalidArgument(format!("Expected a parameter of type {}, this is a parameter with type {}", other_i.kind(), i.kind())))
                            }
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Expected a parameter of type {}, this is a parameter with type {}",
                                other, self
                            )))
                        }
                    }
                }
            }
            GenericKind::Interface(i) => match i {
                InterfaceGenericKind::Dimensionality(d) => {
                    if let GenericKind::Interface(InterfaceGenericKind::Dimensionality(other_d)) =
                        other
                    {
                        if d.condition().satisfies(other_d.condition()) {
                            Ok(())
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Condition \"{}\" is more permissive than condition \"{}\"",
                                d.describe_condition(),
                                other_d.describe_condition()
                            )))
                        }
                    } else {
                        Err(Error::InvalidArgument(format!(
                            "Expected a parameter of type {}, this is a parameter with type {}",
                            other, self
                        )))
                    }
                }
            },
        }
    }
}

impl fmt::Display for GenericKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericKind::Behavioral(b) => write!(f, "Behavioral({})", b),
            GenericKind::Interface(i) => write!(f, "Interface({})", i),
        }
    }
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
    doc: Option<String>,
}

impl GenericParameter {
    pub fn try_new(
        name: impl TryResult<Name>,
        kind: impl TryResult<GenericKind>,
        default_value: impl TryResult<GenericParamValue>,
    ) -> Result<Self> {
        let default_value: GenericParamValue = default_value.try_result()?;
        let r = Self {
            name: name.try_result()?,
            kind: kind.try_result()?,
            default_value: default_value.reduce(),
            doc: None,
        };
        if !r.default_value().is_fixed() {
            Err(Error::InvalidArgument(format!(
                "Default value ({}) is not fixed.",
                r.default_value(),
            )))
        } else if r.valid_value(r.default_value().clone())? {
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

impl Document for GenericParameter {
    fn doc(&self) -> Option<&String> {
        self.doc.as_ref()
    }
}

impl Documents for GenericParameter {
    fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::generics::{
        behavioral::integer::IntegerGeneric, param_value::combination::GenericParamValueOps,
    };

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

        let param2_nat = GenericParameter::try_new("b", IntegerGeneric::natural(), 0)?;
        let param2_pos = GenericParameter::try_new("b", IntegerGeneric::positive(), 1)?;
        let param2_int = GenericParameter::try_new("b", IntegerGeneric::integer(), 0)?;
        let param2_dim = GenericParameter::try_new("b", InterfaceGenericKind::dimensionality(), 2)?;

        assert_eq!(param.valid_value(param2_nat)?, true);
        assert_eq!(param.valid_value(param2_pos)?, true);
        assert_eq!(param.valid_value(param2_int.clone())?, true);
        assert_eq!(param.valid_value(param2_dim.clone())?, true);

        let math_combinaton = param2_dim
            .g_add(2)?
            .g_sub(1)?
            .g_mul(param2_int)?
            .g_mod(3)?
            .g_negative()?;

        assert_eq!(param.valid_value(math_combinaton)?, true);

        Ok(())
    }
}
