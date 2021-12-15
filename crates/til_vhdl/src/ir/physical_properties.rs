#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortDirection {
    Source,
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalProperties {
    direction: PortDirection,
}
