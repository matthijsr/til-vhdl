use std::{any::type_name, fmt, str::FromStr};
use tydi_common::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericCondition {
    Gt(String),
    Lt(String),
    GtEq(String),
    LtEq(String),
}

impl fmt::Display for GenericCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericCondition::Gt(s) => write!(f, "> {}", s),
            GenericCondition::Lt(s) => write!(f, "< {}", s),
            GenericCondition::GtEq(s) => write!(f, ">= {}", s),
            GenericCondition::LtEq(s) => write!(f, "<= {}", s),
        }
    }
}

impl TryFrom<&str> for GenericCondition {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::from_str(value)
    }
}

impl FromStr for GenericCondition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with(">=") {
            Ok(GenericCondition::GtEq(s[2..].trim().to_string()))
        } else if s.starts_with("<=") {
            Ok(GenericCondition::LtEq(s[2..].trim().to_string()))
        } else if s.starts_with('>') {
            Ok(GenericCondition::Gt(s[1..].trim().to_string()))
        } else if s.starts_with('<') {
            Ok(GenericCondition::Lt(s[1..].trim().to_string()))
        } else {
            Err(Error::InvalidArgument(format!("Cannot parse condition from string \"{}\". Conditions must start with >, <, >=, or <=.", s)))
        }
    }
}

impl GenericCondition {
    pub fn gt(val: impl Into<String>) -> Self {
        Self::Gt(val.into())
    }

    pub fn lt(val: impl Into<String>) -> Self {
        Self::Lt(val.into())
    }

    pub fn gteq(val: impl Into<String>) -> Self {
        Self::GtEq(val.into())
    }

    pub fn lteq(val: impl Into<String>) -> Self {
        Self::LtEq(val.into())
    }

    pub fn val(&self) -> &str {
        match self {
            GenericCondition::Gt(s)
            | GenericCondition::Lt(s)
            | GenericCondition::GtEq(s)
            | GenericCondition::LtEq(s) => s,
        }
    }

    pub fn parse_val<T: FromStr<Err = impl fmt::Display>>(&self) -> Result<T> {
        match self {
            GenericCondition::Gt(s)
            | GenericCondition::Lt(s)
            | GenericCondition::GtEq(s)
            | GenericCondition::LtEq(s) => match T::from_str(s) {
                Ok(val) => Ok(val),
                Err(err) => Err(Error::ProjectError(format!(
                    "A condition has an unsuitable value for type {}, value was: {} - Error: {}",
                    type_name::<T>(),
                    s,
                    err
                ))),
            },
        }
    }

    /// Verify against inclusive min and max
    pub fn verify_min_max<T: FromStr<Err = impl fmt::Display> + PartialOrd + fmt::Display>(
        &self,
        param_type: &str,
        min: T,
        max: T,
    ) -> Result<()> {
        let val = self.parse_val::<T>()?;
        if min > max {
            return Err(Error::ComposerError(format!(
                "Min ({}) > Max ({}) for type {}. Please report this error.",
                min, max, param_type
            )));
        }
        let invalid_condition = || {
            Err(Error::InvalidArgument(format!(
                "Condition ({}) is outside of the range of the given type ({}: {}..={})",
                self, param_type, min, max
            )))
        };
        match self {
            GenericCondition::Gt(_) => {
                if val >= max {
                    invalid_condition()
                } else {
                    Ok(())
                }
            }
            GenericCondition::Lt(_) => {
                if val <= min {
                    invalid_condition()
                } else {
                    Ok(())
                }
            }
            GenericCondition::GtEq(_) => {
                if val > max {
                    invalid_condition()
                } else {
                    Ok(())
                }
            }
            GenericCondition::LtEq(_) => {
                if val < min {
                    invalid_condition()
                } else {
                    Ok(())
                }
            }
        }
    }
}
