#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PortDirection {
    Source,
    Sink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PhysicalProperties {
    direction: PortDirection,
}
