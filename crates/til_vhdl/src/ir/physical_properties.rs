#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterfaceDirection {
    /// Indicates this port is a Source (generates output)
    Out,
    /// Indicates this port is a Sink (takes input)
    In,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalProperties {
    direction: InterfaceDirection,
}

impl PhysicalProperties {
    pub fn new(direction: InterfaceDirection) -> Self {
        PhysicalProperties { direction }
    }

    pub fn origin(&self) -> InterfaceDirection {
        self.direction
    }
}
