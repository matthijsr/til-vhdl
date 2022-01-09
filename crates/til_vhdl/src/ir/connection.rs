use std::convert::{TryFrom, TryInto};

use tydi_common::{error::Error, name::Name};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceReference {
    streamlet_instance: Name,
    port: Name,
}

impl InterfaceReference {
    pub fn new(streamlet_instance: Name, port: Name) -> Self {
        InterfaceReference {
            streamlet_instance,
            port,
        }
    }

    pub fn streamlet_instance(&self) -> &Name {
        &self.streamlet_instance
    }

    pub fn port(&self) -> &Name {
        &self.port
    }
}

impl TryFrom<(&str, &str)> for InterfaceReference {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        Ok(InterfaceReference::new(
            value.0.try_into()?,
            value.1.try_into()?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connection {
    source: InterfaceReference,
    sink: InterfaceReference,
}

impl Connection {
    pub(crate) fn new(source: InterfaceReference, sink: InterfaceReference) -> Self {
        Connection { source, sink }
    }

    pub fn source(&self) -> &InterfaceReference {
        &self.source
    }

    pub fn sink(&self) -> &InterfaceReference {
        &self.sink
    }
}
