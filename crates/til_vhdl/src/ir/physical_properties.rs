#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Indicates this port is a Source (generates output)
    Source,
    /// Indicates this port is a Sink (takes input)
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalProperties {
    direction: PortDirection,
}

impl PhysicalProperties {
    pub fn new(direction: PortDirection) -> Self {
        PhysicalProperties { direction }
    }

    pub fn direction(&self) -> PortDirection {
        self.direction
    }
}
