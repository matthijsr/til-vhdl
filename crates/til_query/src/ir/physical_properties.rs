use core::fmt;

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
    direction: InterfaceDirection,
}

impl PhysicalProperties {
    pub fn new(direction: InterfaceDirection) -> Self {
        PhysicalProperties { direction }
    }

    pub fn direction(&self) -> InterfaceDirection {
        self.direction
    }
}

impl From<InterfaceDirection> for PhysicalProperties {
    fn from(direction: InterfaceDirection) -> Self {
        PhysicalProperties { direction }
    }
}
