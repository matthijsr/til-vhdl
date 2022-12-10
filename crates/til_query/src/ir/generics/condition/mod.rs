use std::{any::type_name, fmt, str::FromStr};
use tydi_common::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericCondition {
    Gt(String),
    Lt(String),
    GtEq(String),
    LtEq(String),
}

impl GenericCondition {
    pub fn parse_val<E: fmt::Display, T: FromStr<Err = E>>(&self) -> Result<T> {
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
}
