use std::{borrow::Borrow, collections::BTreeMap, convert::TryInto};

use tydi_common::{
    error::{Error, Result, TryResult},
    name::Name,
};
use tydi_intern::Id;

use super::{
    connection::InterfaceReference, physical_properties::InterfaceDirection, Connection, GetSelf,
    Implementation, Interface, InternSelf, Ir, Streamlet,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    ports: BTreeMap<Name, Id<Interface>>,
    streamlet_instances: BTreeMap<Name, Id<Streamlet>>,
    connections: Vec<Connection>,
    implementations: BTreeMap<Name, Id<Implementation>>,
}

impl Context {
    pub fn new(ports: impl Into<BTreeMap<Name, Id<Interface>>>) -> Self {
        Context {
            ports: ports.into(),
            streamlet_instances: BTreeMap::new(),
            connections: vec![],
            implementations: BTreeMap::new(),
        }
    }

    pub fn port_ids(&self) -> &BTreeMap<Name, Id<Interface>> {
        &self.ports
    }

    pub fn ports(&self, db: &dyn Ir) -> BTreeMap<Name, Interface> {
        let mut result = BTreeMap::new();
        for (key, value) in self.port_ids() {
            result.insert(key.clone(), value.get(db));
        }
        result
    }

    pub fn streamlet_instances(&self) -> &BTreeMap<Name, Id<Streamlet>> {
        &self.streamlet_instances
    }

    pub fn try_add_connection(
        &mut self,
        db: &dyn Ir,
        left: impl TryResult<InterfaceReference>,
        right: impl TryResult<InterfaceReference>,
    ) -> Result<()> {
        let left = left.try_result()?;
        let right = right.try_result()?;

        struct InterfaceAndContext {
            on_streamlet: bool,
            interface: Interface,
        }

        let get_port = |i: &InterfaceReference| match i.streamlet_instance() {
            Some(streamlet_instance) => self
                .try_get_streamlet_instance(streamlet_instance)
                .and_then(|x| {
                    Ok(InterfaceAndContext {
                        on_streamlet: true,
                        interface: x.get(db).try_get_port(db, i.port())?,
                    })
                }),
            None => match self.port_ids().get(i.port()) {
                Some(port) => Ok(InterfaceAndContext {
                    on_streamlet: false,
                    interface: port.get(db),
                }),
                None => Err(Error::InvalidArgument(format!(
                    "No port with name {} exists within this context",
                    i.port()
                ))),
            },
        };
        let left_i = get_port(&left)?;
        let right_i = get_port(&right)?;
        // Interfaces are on the same layer if they both either belong to the context or to a streamlet instance
        let same_layer = left_i.on_streamlet == right_i.on_streamlet;

        if left_i.interface.stream_id() == right_i.interface.stream_id()
            // If the interfaces are on the same layer, their directions should be opposite.
            // If they are not on the same layer, their directions should be the same.
            && same_layer == (left_i.interface.direction() != right_i.interface.direction())
        {
            let (source, sink) = match left_i.interface.direction() {
                // If left_interface belongs to a streamlet instance, Out means it's a Source
                InterfaceDirection::Out if left_i.on_streamlet => (left, right),
                // Otherwise, it belongs to the context, and is a Sink
                InterfaceDirection::Out => (right, left),
                // Likewise, In means it is a Sink if left_interface is a streamlet instance
                InterfaceDirection::In if left_i.on_streamlet => (right, left),
                // But it is a Source if it belongs to the context
                InterfaceDirection::In => (left, right),
            };

            self.connections.push(Connection::new(source, sink));
            Ok(())
        } else {
            Err(Error::InvalidTarget(format!(
                "The ports {} and {} are incompatible",
                left, right
            )))
        }
    }

    pub fn try_add_streamlet_instance(
        &mut self,
        name: impl TryResult<Name>,
        streamlet: Id<Streamlet>,
    ) -> Result<()> {
        let name = name.try_result()?;
        if self.streamlet_instances().contains_key(&name) {
            Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} already exists in this context",
                name
            )))
        } else {
            self.streamlet_instances.insert(name, streamlet);
            Ok(())
        }
    }

    pub fn try_get_streamlet_instance(
        &self,
        name: &Name,
    ) -> Result<Id<Streamlet>> {
        match self.streamlet_instances().get(name) {
            Some(streamlet) => Ok(*streamlet),
            None => Err(Error::InvalidArgument(format!(
                "A streamlet instance with name {} does not exist in this context",
                name
            ))),
        }
    }
}

impl From<&Streamlet> for Context {
    fn from(streamlet: &Streamlet) -> Self {
        Context::new(streamlet.port_ids().clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::logical::logicaltype::Stream, ir::Database, test_utils::test_stream_id};

    use super::*;

    #[test]
    fn try_add_connection() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut context = Context::from(&streamlet);
        context.try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        context.try_add_connection(db, "a", ("instance", "a"))?;

        Ok(())
    }

    #[test]
    fn try_get_streamlet_instance() -> Result<()> {
        let _db = Database::default();
        let db = &_db;
        let stream = test_stream_id(db)?;
        let streamlet = Streamlet::try_new(
            db,
            "a",
            vec![
                ("a", stream, InterfaceDirection::In),
                ("b", stream, InterfaceDirection::Out),
            ],
        )?;
        let mut context = Context::from(&streamlet);
        context.try_add_streamlet_instance("instance", streamlet.with_implementation(db, None))?;
        context.try_get_streamlet_instance(&("instance".try_result()?))?;

        Ok(())
    }
}
