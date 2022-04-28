use core::fmt;

use tydi_common::name::Name;

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
    domain: Option<Name>,
    direction: InterfaceDirection,
}

impl PhysicalProperties {
    pub fn new(domain: Name, direction: InterfaceDirection) -> Self {
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

    /// Get a reference to the physical properties's domain.
    #[must_use]
    pub fn domain(&self) -> Option<&Name> {
        self.domain.as_ref()
    }

    pub fn set_domain(&mut self, domain: Name) {
        self.domain = Some(domain);
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

impl From<(Name, InterfaceDirection)> for PhysicalProperties {
    fn from((name, direction): (Name, InterfaceDirection)) -> Self {
        PhysicalProperties {
            domain: Some(name),
            direction,
        }
    }
}
