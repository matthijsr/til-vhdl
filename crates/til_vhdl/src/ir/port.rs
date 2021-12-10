use tydi_intern::Id;

use super::{PhysicalProperties, Stream};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Port {
    stream: Id<Stream>,
    physical_properties: PhysicalProperties,
}
