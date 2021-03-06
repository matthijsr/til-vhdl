use core::fmt;

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
};

pub type Domain = Name;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterfaceDirection {
    /// Indicates this port is a Source (generates output)
    Out,
    /// Indicates this port is a Sink (takes input)
    In,
}

impl fmt::Display for InterfaceDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceDirection::In => write!(f, "in"),
            InterfaceDirection::Out => write!(f, "out"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalProperties {
    domain: Option<Domain>,
    direction: InterfaceDirection,
}

impl PhysicalProperties {
    pub fn new(domain: Domain, direction: InterfaceDirection) -> Self {
        PhysicalProperties {
            domain: Some(domain),
            direction,
        }
    }

    pub fn new_direction(direction: InterfaceDirection) -> Self {
        PhysicalProperties {
            domain: None,
            direction,
        }
    }

    pub fn direction(&self) -> InterfaceDirection {
        self.direction
    }

    /// When `None`, this refers to the Default domain instead
    pub fn domain(&self) -> Option<&Domain> {
        self.domain.as_ref()
    }

    pub fn set_domain(&mut self, domain: Option<Domain>) {
        self.domain = domain;
    }
}

impl From<InterfaceDirection> for PhysicalProperties {
    fn from(direction: InterfaceDirection) -> Self {
        PhysicalProperties {
            domain: None,
            direction,
        }
    }
}

impl From<(Option<Name>, InterfaceDirection)> for PhysicalProperties {
    fn from((name, direction): (Option<Name>, InterfaceDirection)) -> Self {
        PhysicalProperties {
            domain: name,
            direction,
        }
    }
}

impl From<(Name, InterfaceDirection)> for PhysicalProperties {
    fn from((name, direction): (Name, InterfaceDirection)) -> Self {
        PhysicalProperties {
            domain: Some(name),
            direction,
        }
    }
}

impl TryFrom<(&str, InterfaceDirection)> for PhysicalProperties {
    type Error = Error;

    fn try_from((name, direction): (&str, InterfaceDirection)) -> Result<Self> {
        Ok(PhysicalProperties {
            domain: Some(name.try_result()?),
            direction,
        })
    }
}
