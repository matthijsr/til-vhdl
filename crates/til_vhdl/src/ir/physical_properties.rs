#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// Indicates this port is a Source (generates output)
    Source,
    /// Indicates this port is a Sink (takes input)
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalProperties {
    origin: Origin,
}

impl PhysicalProperties {
    pub fn new(direction: Origin) -> Self {
        PhysicalProperties { origin: direction }
    }

    pub fn origin(&self) -> Origin {
        self.origin
    }
}
