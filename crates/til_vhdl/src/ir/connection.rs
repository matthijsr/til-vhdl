use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

use tydi_common::{
    error::{Error, Result},
    name::Name,
    traits::Reverse,
};

/// References a specific interface (port) within a `Context`,
/// the streamlet_instance may be left blank to refer to the context's own
/// ports, rather than that of a specific streamlet instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceReference {
    streamlet_instance: Option<Name>,
    port: Name,
}

impl InterfaceReference {
    /// Using `None` for `streamlet_instance` indicates you referencing a context's own ports,
    /// rather than a port of a streamlet instance within said context.
    pub fn new(streamlet_instance: Option<Name>, port: Name) -> Self {
        InterfaceReference {
            streamlet_instance,
            port,
        }
    }

    pub fn streamlet_instance(&self) -> &Option<Name> {
        &self.streamlet_instance
    }

    pub fn port(&self) -> &Name {
        &self.port
    }

    pub fn is_local(&self) -> bool {
        match self.streamlet_instance() {
            Some(_) => false,
            _ => true,
        }
    }
}

impl TryFrom<&str> for InterfaceReference {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(InterfaceReference::new(None, value.try_into()?))
    }
}

impl TryFrom<(&str, &str)> for InterfaceReference {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self> {
        Ok(InterfaceReference::new(
            if value.0.trim().is_empty() {
                None
            } else {
                Some(value.0.try_into()?)
            },
            value.1.try_into()?,
        ))
    }
}

impl fmt::Display for InterfaceReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.streamlet_instance() {
            Some(streamlet_instance) => write!(f, "{}.{}", streamlet_instance, self.port()),
            None => write!(f, "{}", self.port()),
        }
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

    pub fn is_local_to_local(&self) -> bool {
        self.source().is_local() && self.sink().is_local()
    }
}

impl Reverse for Connection {
    fn reverse(&mut self) {
        std::mem::swap(&mut self.sink, &mut self.source)
    }
}
